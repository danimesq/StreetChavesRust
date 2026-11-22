# Delphi source reconstruction and reproducible binary research

## Long-term research objective

Investigate whether the original **Street Chaves 1.5.2A** executable can be reconstructed at two distinct levels. These are related research tracks, but success in one does not imply success in the other.

### Track A: Delphi source reconstruction

Reconstruct the original program's source-level structure and semantics as far as the available evidence permits.

Research may include:
- identifying the exact Delphi compiler/toolchain, RTL, VCL, DelphiX and relevant component versions;
- recovering unit, class, form, method and resource structure from PE metadata, RTTI, resources, strings, disassembly and decompiler evidence;
- mapping recovered routines and data structures to the existing Pascal reconstruction and observed runtime behavior;
- documenting which source details are directly recovered, inferred, approximated or still unknown;
- producing a maintainable Delphi-compatible source reconstruction where legally and technically possible.

A reconstructed source tree is not automatically the original source. Decompiled pseudocode, inferred declarations and recreated project files must remain labeled as such.

### Track B: byte-for-byte executable reproduction

Separately investigate whether a rebuilt executable can be made byte-identical to the supplied original binary.

The strongest acceptance criterion is:

`SHA256(rebuilt.exe) == SHA256(original.exe)`

This likely requires much more than equivalent gameplay or equivalent source logic. Investigate exact compiler and linker versions/options, unit/package versions, resource compiler behavior, input resource bytes, object/link order, alignment, timestamps and other PE metadata, deterministic build settings, and any post-build transformations.

Record the original executable's provenance and immutable hash before comparison. Do not claim byte identity from matching behavior, matching PE structure, or a successful launch.

## Evidence and acceptance rules

- Keep source reconstruction and binary reproduction as separate workstreams with separate status and exit criteria.
- Preserve the distinction between original source, recovered/decompiled material, inferred reconstruction and newly written compatibility code.
- Record tool names, versions, commands, environment, input hashes and output hashes for every reproducible experiment.
- Treat a non-identical binary as a useful result: classify the differing PE regions and determine whether differences are semantic, resource-related, toolchain-related or metadata-only.
- Do not let this long-term objective block the first playable Rust baseline or the staged original-executable gameplay validation.
- Respect source, executable and asset provenance/licensing constraints. Technical access does not itself grant redistribution rights.

## Suggested investigation sequence

1. Reconfirm the canonical original executable and archive hashes.
2. Identify Delphi/compiler/RTL/component fingerprints with reproducible PE and resource reports.
3. Build a symbol, unit, form, resource and call-graph evidence map.
4. Reconstruct source units incrementally, attaching confidence and evidence to each recovered element.
5. Establish a controlled build environment matching the identified toolchain.
6. Compare rebuilt and original PE files structurally, then by section/resource bytes and finally by whole-file SHA-256.
7. Document irreducible differences and the evidence needed to resolve them.

**Status: Planned, long-term research.** No Delphi source reconstruction or byte-identical rebuild is claimed to exist yet.
