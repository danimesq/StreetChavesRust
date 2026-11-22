# Original executable reverse-engineering plan

## Purpose and scope

This document records the initial static-inspection baseline and the next investigation sequence for the original **Street Chaves 1.5.2A** Windows executable. It is a research plan, not a claim that the executable's runtime behavior has been validated.

The original executable is a third, independent evidence source alongside the Pascal/SDL2 and C17/SDL3 projects. Keep observations from each source separate. Agreement between the SDL projects is useful cross-port evidence, but it does not establish original-game behavior.

The project delivery order remains explicit:

1. Build an independently playable Rust baseline from the best documented evidence available.
2. Only after that baseline exists, begin the final compatibility stage by pairing Rust with the original executable.

Original-executable parity is not a prerequisite for the first playable Rust build.

## Artifact and evidence ledger

| Item | Preliminary result | Evidence status |
|---|---|---|
| Package | `StreetChaves_1.5.2A.zip` supplied for research | User-provided artifact; preserve original |
| Executable | `Chaves.exe` | Located in package |
| Format | PE32, Intel 80386, Windows GUI | Static-inspection result; reproduce and record tool/version |
| Size | 1,897,984 bytes | Preliminary inspection |
| SHA-256 | `bf3a9f361ea506056a189fc3f453bc5a9a9de08a6f1af4c53f57151e6792bee8` | Preliminary inspection; recompute before relying on it |
| Entry point | `0x004AF6F0` | Preliminary PE inspection |
| Image base | `0x00400000` | Preliminary PE inspection |
| Sections | `CODE`, `DATA`, `BSS`, `.idata`, `.tls`, `.rdata`, `.reloc`, `.rsrc` | Preliminary PE inspection |
| Managed runtime | No CLR indication observed | Preliminary static result |
| Debug/export symbols | No useful debug or export symbols observed | Preliminary static result |

The package inventory previously reported 370 ZIP entries: 361 files and 9 directories, including 251 BMP files, 103 WAV files, the executable, and `STREET CHAVES.txt`. Re-run the inventory and record exact paths and hashes in a machine-generated manifest before using individual assets as evidence.

The package text identifies the release as version 1.5.2A and describes controls/options. Treat these as package-declared facts, not independent proof of runtime behavior or redistribution rights.

## Preliminary implementation clues

The first static pass reported strings and import/reference clues consistent with a Borland Delphi/VCL application using DelphiX and legacy DirectX interfaces.

Observed string families included:

- Delphi/VCL identifiers such as `TObject`, `TInterfacedObject`, `TFORM1`, `EAccessViolation`, and `PACKAGEINFO`.
- DelphiX-style components such as `TDXTimer`, `TDXSound`, `TDXWaveList`, `TDXImageList`, `TDirectDraw`, and `TDirectSound`.
- DirectX-related names including DirectDraw, DirectSound, DirectInput, and Direct3D Retained Mode.
- Win32 imports/references including `LoadLibraryA` and `GetProcAddress`, alongside references to DirectX creation entry points.

These clues suggest dynamic loading may be involved for some DirectX APIs. They do not yet prove the exact compiler version, component versions, initialization order, or which API path is used at runtime. Confirm each finding with reproducible PE/import/resource inspection and code cross-references.

Gameplay-related strings reported in the first pass include:

```text
DXTimer1Timer
carregaranimacao
desenhaanimacoes
sequenciagolpe
defendergolpe
mostragolpe
ChavesGolpes
```

These are search anchors only. A string's presence does not establish a function boundary or its semantic responsibility. The next step is to locate references to these strings, identify the associated code/data structures, and follow control flow.

The first pass also reported references to `Chaves.dat`, while no such file appeared in the ZIP extension inventory. This remains an open question: it may be generated, supplied separately, or referenced by an optional path. Do not assume the package is complete or that the file is required until file-open call sites and runtime behavior are traced.

## Investigation sequence

### 1. PE and toolchain

- Recompute the executable SHA-256 and record the exact archive/member path.
- Record PE headers, section virtual/raw sizes, characteristics, entry point, image base, subsystem, data directories, imports, delay imports, relocations, and resource directory.
- Identify compiler, linker, RTL, VCL, DelphiX, and DirectX fingerprints where evidence supports it.
- Record tool names and versions, commands, and output hashes so the inspection can be repeated.
- Determine whether symbols, RTTI, form resources, or package metadata expose useful class and method names.

**Deliverable:** reproducible PE report and a short toolchain-identification note with confidence levels.

### 2. Resources

- Enumerate every PE resource by type, name/ID, language, size, and hash.
- Extract RCDATA and inspect its internal structure before assigning meaning.
- Inspect form resources and component properties for timer intervals, event handlers, image lists, sound lists, and configuration.
- Extract strings with offsets and encoding labels; distinguish resource strings from code/data strings.
- Cross-reference embedded resources with external package assets without assuming they are interchangeable.

**Deliverable:** resource catalog, extraction manifest, and links from resource entries to relevant code references.

### 3. External assets

- Generate a path/size/hash manifest for all BMP, WAV, music, text, and other package files.
- Compare image and audio assets with the Pascal and C projects by hashes first, then dimensions/content where hashes differ.
- Record sprite-sheet/frame relationships, sound identity, and naming differences only when supported by inspection.
- Keep provenance and licensing separate from technical similarity. Do not copy or redistribute original assets merely because they are present in the user-provided package.
- Investigate missing or dynamically generated data such as the reported `Chaves.dat` reference.

**Deliverable:** asset comparison matrix with exact paths, hashes, dimensions/durations, provenance, and unresolved mappings.

### 4. Gameplay and code cross-references

Start with the reported anchors:

1. `DXTimer1Timer`: locate its event-handler reference and determine what work it schedules. Trace call order, timer interval, tick counters, pause behavior, and any dependence on rendering or wall-clock time.
2. `sequenciagolpe`: trace command recognition, sequence state, timing windows, and move selection.
3. `defendergolpe`: trace guard eligibility, direction/state checks, hit qualification, and resulting state changes.
4. `carregaranimacao` and `desenhaanimacoes`: separate resource loading, animation-state advancement, frame selection, and drawing.

Then follow the resulting call graph into:

- input acquisition and logical command mapping;
- fighter state and action transitions;
- movement, collision, hit/hurt geometry, damage, guard, knockback, hit-stop, and combo handling;
- AI decisions and random-number generation;
- effects/projectiles and their lifecycle;
- round/game flow, pause, and timers;
- audio/visual event dispatch.

Use cross-references and data-flow analysis rather than assigning semantics from names alone. Record addresses/RVAs, function boundaries, callers/callees, relevant structures, branch conditions, and confidence. If a function boundary or type is uncertain, mark it as inferred.

**Deliverable:** annotated call graph and subsystem notes that can be compared against the two SDL implementations.

### 5. Golden fixtures and differential validation

Golden fixtures belong to the later compatibility stage, after the first playable Rust baseline exists. Prepare their schema and capture protocol earlier, but do not make original-executable parity a blocker for the baseline.

Each fixture should record:

- executable SHA-256 and package identity;
- OS/environment and relevant display/input settings;
- initial scenario and fighter selections;
- exact logical inputs and their timing;
- observable tick/frame checkpoints;
- screenshots/audio captures when useful, clearly separated from hidden-state claims;
- expected observations, repeatability, and confidence.

Prioritize idle/movement/jump/landing, representative normal and special moves, guard/hit/damage, knockback/hit-stop, projectiles/effects, pause, round transitions, and timer boundaries. Compare original observations with Pascal, C, and Rust independently. A screenshot alone cannot establish hidden logical state.

**Deliverable:** versioned fixture set and differential matrix identifying the first divergence without silently changing expected results.

## Long-term source and binary reconstruction goals

The broader long-term goal is documented separately in [Delphi source reconstruction and reproducible binary research](delphi-source-and-reproducible-binary.md). Keep its two tracks distinct:

- **Source reconstruction:** recover Delphi units, classes, forms, methods, resources and semantics, labeling decompiled or inferred material rather than presenting it as original source.
- **Byte-identical rebuild:** investigate whether the original executable can be rebuilt with an exact whole-file SHA-256 match. This requires identifying the toolchain and build inputs, and may be blocked by unavailable compiler versions, resources, metadata or provenance.

A source reconstruction can be useful without producing an identical executable. A behaviorally equivalent or structurally similar executable is not byte-identical. No success on either track is claimed by this preliminary plan.

## Evidence and safety rules

- Label each statement as direct static observation, runtime observation, source observation, inference, or unknown.
- Keep addresses tied to the exact executable hash; do not carry addresses across binaries without revalidation.
- Never infer original-game behavior solely from DelphiX component names or string literals.
- Static inspection does not prove runtime execution paths.
- Do not run unknown executables on the host environment. Any runtime capture should use an intentionally isolated, documented environment.
- Preserve the user-provided original archive and executable unchanged; work on extracted copies.
- Keep original code/assets out of Rust deliverables unless provenance and license review explicitly permits reuse.
- Record unresolved contradictions instead of choosing the interpretation that is easiest to implement.

## Immediate next tasks

- [ ] Recompute and independently verify the executable hash.
- [ ] Save PE inspection commands, tool versions, and full machine-readable output.
- [ ] Extract and enumerate PE resources, especially the large RCDATA entry.
- [ ] Produce string-offset and cross-reference reports for the four gameplay anchors.
- [ ] Locate the timer event-handler code and map its callers/callees.
- [ ] Build the first call-graph slice from timer to input, fighter update, collision, and rendering.
- [ ] Generate external asset hashes and compare them with both SDL repositories.
- [ ] Add unresolved findings to the evidence register.
