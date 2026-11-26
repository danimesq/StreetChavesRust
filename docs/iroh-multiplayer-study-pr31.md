# Iroh multiplayer networking study

**Status:** architecture study and implementation proposal. No networking code or dependencies are added by this document.

**Research snapshot:** 2026-11-25. Iroh and its protocol crates evolve independently; version/API recommendations must be revalidated and pinned when a prototype is approved.

## 1. Executive summary

Iroh is a strong candidate for the **connectivity and encrypted transport layer** of StreetChaves multiplayer. It provides peer identity, QUIC connections, NAT traversal, direct-path establishment where possible, and encrypted relay fallback. It does **not** provide the game rules, matchmaking policy, player authorization, anti-cheat, authoritative simulation, rollback, or a complete multiplayer game protocol.

For StreetChaves, the recommended first architecture is:

1. Keep `FightSimulation` deterministic, headless, and unaware of Iroh.
2. Add a separate Rust networking adapter that owns one Iroh `Endpoint` per process and exposes typed game messages to the application.
3. Start with a **host-authoritative two-player match**. One peer runs the authoritative `FightSimulation`; the other sends logical inputs and receives authoritative state/events.
4. Use Iroh's QUIC connection and streams for session setup, control messages, and reliable transfers. Evaluate QUIC datagrams for latency-sensitive input/state traffic, with explicit sequence numbers, tick numbers, expiry, and recovery rules.
5. Use optional mDNS-based address lookup for LAN discovery. Keep manual invite/EndpointId connection as the simplest baseline. Use internet address lookup and relays for online connectivity.
6. Treat `iroh-gossip` as an optional lobby/presence/discovery tool for groups, not as the transport for every combat frame and not as the authority for a match.
7. Do not implement host migration, rollback netcode, or a public matchmaking service in the first networking milestone. Define the seams so they can be added after measurements.

**Recommendation:** prototype Iroh behind a `chaves-net`-style adapter only after the study is accepted. The current repository is still a deterministic simulation project, not a playable network game; this document does not claim otherwise.

## 2. What Iroh actually provides

Iroh is a Rust networking library built around authenticated QUIC connections between endpoints. An endpoint has a cryptographic identity (`EndpointId`) derived from its public key. A peer connects using the peer identity plus addressing information and an application protocol identifier (ALPN).

The connection process typically works as follows:

1. Each endpoint establishes connectivity to a relay service and learns address information.
2. The peers attempt to establish a direct path, including NAT traversal/hole punching where possible.
3. If direct connectivity cannot be established, encrypted traffic can continue through a relay.
4. Iroh manages the connection path; the application uses QUIC connections, streams, and datagrams.

The relay is **not** a game server. It forwards encrypted traffic and cannot inspect the application payload. It does not validate a move, decide a hit, simulate a frame, or protect the match from a malicious player.

### 2.1 Useful primitives

| Iroh primitive | Potential StreetChaves use | Important boundary |
|---|---|---|
| `Endpoint` / `EndpointId` | Process-level network identity and connection lifecycle | Identity proves possession of a key, not permission to join a particular match |
| `Connection` | An authenticated peer session | Application still defines protocol, authorization, limits, and lifecycle |
| QUIC bidirectional stream | Handshake, match setup, reliable control, resync snapshots | Ordered within that stream; do not put all latency-sensitive traffic behind a large transfer |
| QUIC unidirectional stream | One-way logs, replay transfer, optional telemetry | Sender-created stream; application still frames/version-controls messages |
| QUIC datagrams | Candidate for time-sensitive input/state updates | Unreliable and size/transport constrained; sequence, loss, expiry, and fallback are application concerns |
| Relay | Connectivity fallback when direct paths fail | Adds a network hop and latency; it is not authoritative compute |
| Address lookup | Resolve an EndpointId to usable address information | Discovery is not matchmaking or authorization |
| ALPN / protocol router | Versioned StreetChaves wire protocol dispatch | Must reject unsupported protocol versions explicitly |

QUIC streams are independently multiplexed. A long replay or snapshot transfer should use a different stream from small control messages so one stream's ordering does not unnecessarily delay another. This does not remove congestion, bandwidth, or CPU contention.

### 2.2 Iroh is not the whole multiplayer stack

The project must still design and implement:

- Matchmaking and invite semantics.
- Player slots, role assignment, and match admission.
- A versioned message schema and compatibility policy.
- Tick/input sequencing and validation.
- Authority and conflict resolution.
- State snapshots, replay, reconnect, and resynchronization.
- Rate limits, payload limits, abuse handling, and disconnect policy.
- Latency measurement and telemetry.
- A policy for relay availability, deployment, and operational cost.

## 3. Online connectivity and LAN connectivity

These are related but distinct problems. Iroh can carry the same application protocol over different paths, but discovering a nearby player is not the same as establishing a secure match.

### 3.1 Online play

For internet play, an endpoint can use address lookup and relay-assisted connection establishment. Iroh attempts a direct path and can continue over a relay when direct connectivity is unavailable.

Possible invitation flow:

1. Host creates an endpoint and a match invitation.
2. Host publishes or shares the endpoint's addressing information and a short-lived match ticket.
3. Guest obtains the invitation through an out-of-band channel (copy/paste, QR code, friend system, or a future lobby service).
4. Guest connects using the endpoint identity/address and the StreetChaves ALPN.
5. Host authenticates the peer at the application layer, reserves a player slot, and starts the match only after protocol/version checks pass.

A public EndpointId should not be treated as a secret. The invitation needs a separate authorization mechanism, such as a random, short-lived, single-use match capability bound to the intended match and role. Avoid embedding long-lived secrets in URLs, logs, or replay files.

### 3.2 LAN play

For LAN play, investigate the current `iroh-mdns-address-lookup` crate and its compatibility with the selected Iroh release. It advertises/discovers endpoint address information on the local network using mDNS-like discovery.

Recommended user experience:

- A **Find LAN Games** action browses local discovery results.
- The host advertises a game-specific service or metadata record where supported by the chosen discovery implementation.
- The client displays candidates, then connects through Iroh and performs the exact same authenticated application handshake as online play.
- Manual EndpointId/invite entry remains available as a fallback.

Important caveats:

- mDNS depends on multicast being permitted by the LAN, access point, operating system, and firewall.
- Guest Wi-Fi/client isolation, VLAN separation, VPNs, and some enterprise networks can prevent discovery even when internet connectivity works.
- A discovered endpoint is only a candidate. It is not trusted until the peer identity and match authorization are validated.
- Do not assume that every local IPv6 link-local configuration is supported equally by every Iroh version and platform. Test real networks.
- If mDNS fails, the UI should explain that local discovery is unavailable and offer manual invite entry. It must not imply that the game itself cannot play online.

### 3.3 Connectivity matrix

| Scenario | Expected approach | Validation needed |
|---|---|---|
| Same machine / loopback | Direct connection or test transport | Useful for protocol tests, not a substitute for LAN testing |
| Same home LAN | mDNS lookup + direct QUIC path | Multicast, firewall, Wi-Fi isolation, IPv4/IPv6 |
| Different LANs, normal NAT | Relay-assisted setup, then direct hole-punch attempt | Symmetric NAT and router behavior |
| CGNAT / restrictive NAT | Relay fallback may be required | Latency, relay reachability, sustained traffic |
| Relay unavailable | Direct path may still work if address information is available | Explicit failure/retry UX |
| Offline LAN | mDNS/direct path; no public relay required if local addresses are usable | Verify the selected address lookup and endpoint configuration actually work offline |
| VPN / enterprise network | Depends on routes, firewall, multicast and UDP policy | Treat as best-effort; expose diagnostics |

## 4. Which Iroh protocols are relevant?

Iroh's core networking library and its optional protocol crates are separate pieces. The repository should depend only on what the selected use case needs.

### 4.1 `iroh`: primary candidate

This is the core connectivity layer. It should be the only mandatory Iroh dependency in the first network prototype. It provides the endpoint, authenticated connections, QUIC transport, relay integration, and application protocol routing primitives.

### 4.2 `iroh-mdns-address-lookup`: LAN discovery candidate

Use this only for the LAN-browser feature. Keep discovery optional so a build can still connect through a manual invite or online address lookup without mDNS.

Do not confuse endpoint address lookup with game lobby discovery: the game still needs a way to identify which discovered endpoints are actually advertising a StreetChaves match, plus a handshake to verify that claim.

### 4.3 `iroh-gossip`: optional lobby/presence layer

`iroh-gossip` distributes messages among peers subscribed to a shared topic. It uses a membership/broadcast design and requires an initial known peer (bootstrap information) to join a topic.

Potential uses:

- Small friend/lobby presence groups.
- Announcing that a host is waiting for players.
- Propagating room metadata or discovery hints.
- Lightweight group coordination where eventual delivery is acceptable.

Do **not** use gossip as the authoritative combat channel:

- Gossip is a broadcast/swarm abstraction, not a two-player tick protocol.
- Delivery and membership behavior are not a substitute for per-player input validation or a deterministic authority.
- It can relay messages among members and maintain multiple peer connections; that is unnecessary overhead for a single 1v1 match.
- The application must still authenticate, validate, order, expire, and interpret messages.

For the first StreetChaves match, direct host↔guest Iroh connection is simpler. Revisit gossip only if the project adds a real multi-peer lobby or group discovery requirement.

### 4.4 `iroh-blobs`: content/replay transfer, not combat

Content-addressed, verifiable, resumable blob transfer could be useful later for replay files, custom fighter packages, or user-shared assets. It is not a real-time state synchronization protocol and should not be on the combat hot path.

### 4.5 Other ecosystem projects

The Godot Iroh extension is a useful proof that Iroh can be adapted to a game engine's multiplayer peer abstraction. It demonstrates a host/client connection-string flow and then delegates game RPC behavior to Godot. It is a reference architecture, not a drop-in dependency for StreetChaves: it is a Godot extension with its own platform and maturity constraints.

The official Iroh examples also include framed messages over a bidirectional stream and a simple two-peer real-time position demo. These are useful transport examples, but neither proves that Iroh supplies a fighting-game netcode model.

## 5. Multiplayer topology choices

### 5.1 Host-authoritative peer-to-peer (recommended first)

One player hosts the match and runs the authoritative simulation. The other peer sends logical input and receives authoritative results.

Advantages:

- No dedicated game-server fleet is required for an initial release.
- Iroh can connect the two peers directly where possible and relay when necessary.
- There is one source of truth for hit resolution, RNG, timers, and match outcome.
- The architecture matches the existing deterministic `FightSimulation` boundary.

Costs and risks:

- The host has an advantage and can potentially cheat.
- Host disconnect ends the match unless migration is later implemented.
- The host's upload and CPU capacity constrain the session.
- A relay fallback does not remove the host's authority or its trust implications.

This is the best first step for a casual two-player implementation, not a claim of competitive-grade anti-cheat.

### 5.2 Dedicated authoritative server

A headless StreetChaves process runs `FightSimulation` and both players connect to it.

Advantages:

- Neither player is the host authority.
- Server-side validation and match lifecycle are easier to centralize.
- Matchmaking, moderation, and persistent ranking can be added in one place.

Costs:

- Requires deployment, operations, capacity, monitoring, and abuse protection.
- Iroh relays are not a replacement for this server.
- The project must decide whether the server is a normal Iroh endpoint, how clients discover it, and how it authenticates players.

This is the preferred future model if ranked/competitive play or persistent accounts become important.

### 5.3 Symmetric lockstep P2P

Both peers independently run the same deterministic simulation and exchange inputs.

Advantages:

- Potentially low bandwidth.
- Both clients can simulate locally.
- Deterministic replay work already provides useful foundations.

Risks:

- One missing/late input can stall or delay simulation unless prediction/rollback is implemented.
- RNG state, initial state, build/data version, and every relevant ordering rule must match exactly.
- A malicious peer can send fabricated inputs or manipulate its own simulation unless there is a verification model.
- Desync diagnosis and recovery are harder than a single-authority model.

Do not choose this as the first implementation merely because the core is deterministic. Determinism is a prerequisite, not a complete lockstep/rollback solution.

### 5.4 Recommended evolution

| Stage | Topology | Purpose |
|---|---|---|
| 0 | In-process fake transport | Test message protocol and match lifecycle without sockets |
| 1 | Two local processes, host-authoritative | Prove framing, handshake, tick input, snapshots, disconnect |
| 2 | Two LAN machines | Prove mDNS/direct connection and local network UX |
| 3 | Two internet peers | Prove direct path and relay fallback under real NATs |
| 4 | Headless dedicated host | Separate match authority from a player |
| 5 | Optional rollback/lockstep experiments | Only after deterministic snapshots and input history are reliable |
| 6 | Lobby/friend discovery | Add gossip or a separate matchmaking service only when product requirements justify it |

## 6. Proposed StreetChaves network boundary

The network layer must remain an adapter around the game core. The core must not import `iroh`, `tokio`, socket types, endpoint IDs, or transport-specific message types.

Proposed conceptual modules (names are suggestions, not files to create in this PR):

```text
street-chaves-rust/
├── src/
│   ├── fight_simulation.rs       # authoritative deterministic game rules
│   └── ...
├── crates/                       # possible future workspace split
│   ├── chaves-net-protocol/      # versioned, transport-neutral messages
│   ├── chaves-net-iroh/          # Iroh endpoint, QUIC, relay, discovery adapter
│   └── chaves-match-host/        # authority, validation, tick/input scheduling
└── docs/
```

The current repository is a minimal crate, so do not create a multi-crate workspace until a compiling network prototype demonstrates that the split is useful.

### 6.1 Dependency direction

```text
       UI / platform host
              |
              v
       Match coordinator
         /          \
        v            v
 FightSimulation   Net protocol
 (pure game core)     |
                      v
                Iroh adapter
                      |
               QUIC / relay / mDNS
```

The coordinator translates validated network inputs into the existing logical input representation and advances the authoritative simulation. It translates simulation results into transport-neutral snapshots/events. The Iroh adapter only transports encoded protocol messages.

### 6.2 First protocol sketch

Use a dedicated, versioned ALPN such as `street-chaves/match/1` (placeholder; choose and freeze it only when implementation begins). Every message should carry an explicit protocol/schema version and bounded payload length.

Initial message families:

| Message | Direction | Reliability / handling | Purpose |
|---|---|---|---|
| `ClientHello` | Guest → host | Reliable stream | Protocol version, game build, data/roster hash, capabilities |
| `HostHello` | Host → guest | Reliable stream | Match ID, role, tick rate, ruleset, authority identity |
| `JoinProof` | Guest → host | Reliable stream | Single-use invitation proof and requested slot |
| `MatchStart` | Host → guest | Reliable stream | Initial state/checkpoint and start tick |
| `InputFrame` | Guest → host | Datagram candidate; stream fallback to evaluate | Logical buttons/axes, input sequence, intended simulation tick |
| `InputAck` | Host → guest | Datagram or piggybacked snapshot | Highest accepted input sequence/tick |
| `StateSnapshot` | Host → guest | Datagram for small frequent deltas; reliable stream for full checkpoint | Authoritative state and correction |
| `ReliableEvent` | Host → guest | Reliable stream | Round result, pause, rematch, disconnect reason |
| `ResyncRequest` / `ResyncSnapshot` | Both | Reliable stream | Rejoin after packet loss, late join, or state mismatch |
| `Ping` / `Pong` | Both | Datagram candidate | RTT and path monitoring |

This is a design sketch, not an implemented wire format. Serialize with an explicit, versioned schema and strict maximum sizes. Avoid serializing internal Rust structs as an unversioned wire contract.

### 6.3 Tick and input semantics

StreetChaves currently documents a fixed **28 ms logical simulation tick**. Network traffic must target that clock, not the render frame rate or wall-clock frame duration.

An input frame should contain at least:

- Match/session identifier.
- Sender/player slot.
- Monotonic input sequence number.
- Intended simulation tick (or a clearly specified input-delay window).
- Logical button/action state, preserving pressed edges separately from held state.
- Acknowledgment of the latest remote input/state the sender has processed.

The host must reject invalid player slots, stale/future ticks outside policy, duplicate sequences, oversized payloads, unsupported actions, and inputs after the match has ended.

Do not trust a client-provided position, health, hit result, combo count, RNG state, or match outcome. The guest sends intent; the authority runs the actual `FightSimulation`.

### 6.4 Streams vs datagrams

Use separate channels according to semantics:

- **Reliable ordered stream:** handshake, configuration, match start, pause/resume, full state checkpoint, replay transfer, and final results.
- **Datagrams:** candidate for frequent time-sensitive input frames and small state updates where a late packet is less useful than a newer one.
- **Application-level reliability:** if a datagram is lost, the protocol decides whether to resend, supersede it with a newer frame, or recover from a snapshot. QUIC datagrams do not become reliable merely because they share a QUIC connection.
- **Separate streams:** do not send a large replay/snapshot transfer through the same ordered stream used for small control messages.

The correct choice depends on measured latency, loss, message sizes, and Iroh API support in the pinned version. Benchmark both a stream-based baseline and a datagram candidate before choosing.

## 7. Fighting-game-specific concerns

### 7.1 Authority and input delay

A 28 ms simulation tick is approximately 35.7 logical ticks per second. Network arrival does not align perfectly with that clock. The match coordinator needs an explicit policy for late input:

- Buffer inputs for a configurable number of ticks.
- Define when an input becomes committed.
- Decide whether a late input is dropped, delayed, or causes a pause/resync.
- Keep the policy deterministic and visible in replay metadata.

For a first prototype, a small fixed input buffer and host-authoritative snapshots are easier to reason about than rollback.

### 7.2 State snapshots and hashes

Define a stable network snapshot separately from internal Rust memory layout. Include every gameplay-relevant field needed to resume the match: both fighter states, stage state, timers, effects, RNG state, round/match state, and any pending simulation events.

A periodic deterministic state hash can detect divergence. A hash mismatch should trigger diagnostics and, if supported, a full authoritative resync. Do not hash renderer/audio/device state.

### 7.3 Prediction and interpolation

Do not let presentation interpolation alter authoritative simulation state. The remote fighter may be visually interpolated between received snapshots, but combat and collision decisions remain based on the authoritative tick state.

Client-side prediction can be introduced later for local responsiveness, but predicted state must be distinguishable from confirmed authoritative state.

### 7.4 Rollback is a separate project

Rollback would require:

- Snapshotting/restoring the complete deterministic simulation state.
- Recording per-tick inputs for both players.
- Restoring RNG and all hidden counters exactly.
- Re-simulating from a prior tick when late inputs arrive.
- Separating irreversible presentation side effects (sound, particles, UI) from replayable simulation events.
- A policy for maximum rollback depth and peers that exceed it.

The existing deterministic RNG and replay work is a useful foundation, but does not prove the current `FightSimulation` can yet be snapshotted/restored or rolled back safely.

## 8. Security and trust model

Iroh authenticates the remote endpoint's cryptographic identity and encrypts the connection. StreetChaves must still authorize the peer and validate every game message.

Minimum requirements:

- Use a fresh, high-entropy, single-use invitation capability with expiry.
- Bind the capability to a match ID and role; do not treat EndpointId as a password.
- Enforce one peer per reserved slot and define duplicate-connection behavior.
- Bound frame sizes, stream counts, queues, and per-peer message rates.
- Validate all enum values, numeric ranges, tick windows, sequence numbers, and state transitions.
- Never accept client-authored damage, collision results, inventory, health, or victory state.
- Avoid logging private invitation tokens or long-lived endpoint secret keys.
- Define behavior for replayed, duplicated, delayed, reordered, and malformed application messages.
- Treat host authority as trusted only in the casual host-authoritative model; use a dedicated server for stronger competitive integrity.

Encrypted transport protects data in transit and authenticates endpoint keys. It does not prove that a peer is running an unmodified game binary or that its inputs are honest.

## 9. Reliability, latency, and observability

The implementation should expose enough diagnostics to understand the path actually used:

- EndpointId (short display form only).
- Direct vs relay path, when the API exposes it.
- RTT and recent RTT variation.
- Packet/datagram loss estimates where available.
- Input queue depth and oldest pending tick.
- Snapshot age and last acknowledged sequence.
- Reconnect/resync count.
- Disconnect/error category.

Use bounded queues. Under congestion, discard or supersede obsolete transient state rather than allowing unbounded latency to accumulate. Reliable control messages must not be silently dropped.

Test under controlled impairment: latency, jitter, loss, bandwidth limits, connection interruption, and relay-only routing. A localhost test cannot establish online play quality.

## 10. Operational and platform considerations

- **Relay service:** decide whether to use the default public relay infrastructure for prototype testing or operate custom relays later. Document availability, privacy, rate limits, and operating costs before release.
- **Address lookup:** choose a discovery service intentionally. Public DNS/Pkarr-style lookup, mDNS, manual invitation, and a future rendezvous service solve different problems.
- **Secrets:** persist endpoint identity securely where appropriate; do not commit generated secret keys.
- **Async runtime:** Iroh's APIs are asynchronous. Keep async tasks and cancellation/shutdown in the adapter/coordinator, not in `FightSimulation`.
- **Firewall:** document outbound UDP/QUIC and relay fallback behavior; provide useful diagnostics instead of asking users to blindly open inbound ports.
- **WASM/browser:** do not assume native Iroh networking works in browser builds. Browser transport, relay compatibility, and platform support need a separate feasibility check before the WASM/Canvas target is promised.
- **Mobile/console:** verify each target and its OS networking constraints against the exact Iroh release and bindings. Do not infer platform support from the fact that the core crate compiles on desktop.

## 11. Prototype plan and acceptance criteria

A future implementation PR should be split into small, observable milestones.

### Prototype A — transport-neutral protocol

- Define versioned message types and bounded encoding/decoding.
- Unit-test malformed, oversized, duplicate, stale, and unsupported messages.
- Implement a fake in-memory transport.
- Prove the match coordinator can drive a host simulation without importing Iroh.

### Prototype B — Iroh local process connection

- Create one endpoint per process.
- Connect two local processes using a known invitation.
- Negotiate ALPN and protocol version.
- Exchange handshake, logical input, snapshots, and clean shutdown.
- Verify that invalid invitation proof and incompatible versions are rejected.

### Prototype C — LAN

- Add optional mDNS address lookup.
- Discover a host on a second physical machine.
- Verify direct connectivity, offline LAN operation, firewall behavior, and manual-invite fallback.
- Confirm that discovery results do not bypass the authenticated match handshake.

### Prototype D — online

- Test two peers on unrelated internet connections.
- Record direct-path and relay-fallback cases.
- Test CGNAT/restrictive NAT, relay interruption, reconnect, and sustained match duration.
- Measure RTT and input-to-authoritative-response latency.

### Exit criteria before calling multiplayer usable

- No Iroh types or async runtime leak into the deterministic core.
- Same initial state and authoritative input sequence produce the same simulation results in local tests.
- A malformed or unauthorized peer cannot mutate authoritative game state.
- LAN and online flows have explicit, tested failure messages.
- Disconnect and reconnect do not silently resume from stale state.
- Network impairment tests have documented thresholds and reproducible instructions.
- Protocol compatibility and game/data build compatibility are checked before match start.

## 12. Open questions

These require prototype evidence rather than assumptions:

1. Which Iroh release and feature set best match the repository's Rust MSRV and target platforms?
2. Does the selected Iroh release expose the needed QUIC datagram behavior and path metrics through a stable API?
3. Which mDNS/address-lookup crate version works reliably on Windows, Linux, and macOS, and what are its firewall/multicast constraints?
4. Is a 28 ms simulation tick a suitable network input cadence, or should inputs be batched while the simulation remains at 28 ms?
5. What input buffer gives acceptable responsiveness under typical Brazilian home broadband/mobile networks?
6. Is host-authoritative P2P acceptable for casual play, or should the first public online release use a dedicated authoritative process?
7. Which fields are required for a complete serializable `FightSimulation` checkpoint?
8. Can the current simulation be restored and replayed from a checkpoint without hidden external state?
9. What is the intended player identity/account model, if any, beyond endpoint keys and match invitations?
10. Which browser/mobile/console targets are actually required for the first multiplayer release?

## 13. Decision

**Adopt Iroh as the leading candidate for transport/connectivity, subject to a small prototype. Do not adopt Iroh as the game protocol, authority model, or simulation runtime.**

For the first playable network milestone, prefer:

- Host-authoritative 1v1.
- Manual invitation first; optional LAN mDNS discovery second.
- One versioned StreetChaves ALPN.
- Reliable streams for setup/control/checkpoints.
- A measured datagram experiment for time-sensitive inputs.
- Transport-neutral coordinator around the existing deterministic simulation.
- No gossip on the combat hot path.
- No host migration or rollback until their state and replay requirements are proven.

This yields a small, testable path from two local processes to LAN and then internet play without turning the game core into a networking framework.

## References

Primary Iroh sources (consulted 2026-11-25):

- [Iroh crate documentation](https://docs.rs/iroh/latest/iroh/) — endpoint identity, QUIC streams, direct connectivity, relay fallback, and address lookup.
- [Iroh 1.0 roadmap](https://www.iroh.computer/roadmap) — major API/ecosystem changes and the separation of optional protocols.
- [Iroh repository](https://github.com/n0-computer/iroh) — current project architecture and transport overview.
- [Iroh address lookups](https://github.com/n0-computer/iroh-address-lookups) — optional mDNS and Mainline DHT lookup implementations.
- [iroh-mdns-address-lookup README](https://github.com/n0-computer/iroh-address-lookups/tree/main/iroh-mdns-address-lookup) — local-network endpoint discovery.
- [iroh-gossip documentation](https://docs.rs/iroh-gossip/latest/iroh_gossip/) — topic-based swarm messaging.
- [iroh-gossip 0.102.0 release notes](https://www.iroh.computer/blog/iroh-gossip-road-to-1-0) — current protocol status and reliability work.
- [Iroh examples](https://github.com/n0-computer/iroh-examples) — framed messages and application examples.
- [Godot Iroh](https://github.com/tipragot/godot-iroh) — game-engine integration reference; experimental and not a StreetChaves dependency.
- [Godot secure multiplayer guidance](https://docs.godotengine.org/en/latest/tutorials/networking/high_level_multiplayer.html) — general guidance on authoritative game state and untrusted client input.

StreetChaves project sources:

- [Rust architecture proposal](rust-architecture.md) — deterministic core and platform boundary.
- [Fixed 28 ms simulation clock](fixed-timestep-clock.md) — logical tick contract.
- [Headless FightSimulation](headless-fight-simulation.md) — current simulation boundary and limitations.
- [Pascal reference trace harness](pascal-reference-trace-harness.md) — deterministic evidence and replay research.
