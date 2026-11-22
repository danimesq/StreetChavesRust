# Provenance and licensing review

## Purpose

This document is a conservative provenance register for StreetChavesRust. It is not legal advice and does not establish ownership of third-party material. Its purpose is to prevent accidental copying of source code, game assets, extracted data or ROM-derived artwork before the relevant rights and license obligations are understood.

**Default rule:** public availability in a Git repository is not, by itself, permission to reuse every file in that repository.

## Reference projects and declared licenses

| Project | Pinned revision | Root license evidence | Current reuse posture |
|---|---|---|---|
| [OpenStreetChaves](https://github.com/danimesq/OpenStreetChaves) | `e1b025c35ecbc2b14d8635564aafd23c2107d3d1` | Root `LICENSE` is GNU GPL v3.0 | Source-code reuse requires GPL-3.0 compatibility and file-level provenance review. Game assets require separate review. |
| [open-street-chaves](https://github.com/danimesq/open-street-chaves) | `e76ad2188af6842ff6618ecdb8f425bba61c9183` | Root `LICENSE` is GNU GPL v3.0 | Source-code reuse requires GPL-3.0 compatibility and file-level provenance review. README explicitly excludes original game assets/text from the code license. |
| [street-chaves-hack-sms](https://github.com/danimesq/street-chaves-hack-sms) | `6c439d880efa89ab4fccd4accaa70af743c8cda1` | No root `LICENSE` file found in the pinned tree; `emu/LICENSE.txt` exists for the emulator component | Do not assume repository-wide license. Review each script, dependency, patch, font and artwork source independently. |

The two SDL repositories' GPL-3.0 declarations apply to their covered project code under the authors' stated licensing, not automatically to original Street Chaves assets, text, music, voices, character artwork or other third-party material. The C project README explicitly says its GPL license covers only its code and documentation, never the original game's assets or text.

## Material classes

### 1. Project-authored source code

- Pascal/Lazarus SDL2 source in OpenStreetChaves: declared GPL-3.0 at repository root.
- C17/SDL3 source in open-street-chaves: declared GPL-3.0 at repository root.
- SMS project Python/C tooling: no repository-wide license was found at the pinned revision. A separate `emu/LICENSE.txt` must not be generalized to all files.

**Action:** if StreetChavesRust copies or adapts code from either SDL project, retain source attribution, identify the origin file and commit, mark modifications, and evaluate GPL-3.0 obligations for the resulting distribution. Prefer independent reimplementation from documented behavior where that is the intended architecture, while recognizing that clean-room process does not erase copyright concerns around expressive material or copied tables.

### 2. Original PC game assets and extracted resources

The original game is identified by the C project's reverse-engineering documents as Street Chaves 1.5A (2004), associated with CyberGamba. The PC asset set includes fighter strips, portraits, stage backgrounds, effects, animations, voices, sounds, music and in-game text. OpenStreetChaves checks many such resources into its repository. The C project README says it does not include those assets and requires the user to supply their own original game folder and executable.

The C project's README explicitly states that its GPL code license does not cover original game assets or text. The Pascal repository's root GPL file does not, by itself, prove that every checked-in extracted resource was authored by or licensed from the repository maintainer.

**Action:** treat all original PC assets and extracted text/audio as **rights unresolved** until a per-file or per-class permission/license basis is documented. Do not copy these assets into StreetChavesRust, package them, mirror them, or generate distributable derivatives based solely on their presence in OpenStreetChaves.

### 3. Original executable and reverse-engineering artifacts

The C project's `docs/spec/` describes analysis of `Chaves.exe`, including static binary inspection, Delphi metadata, Ghidra pseudocode, extracted DFM resources and generated tables. The spec distinguishes static facts, decompiled behavior, inference and unknowns.

Reverse-engineering notes and independently written descriptions are not the same thing as redistributing the executable. However, extracted resources, verbatim text, decompiled expressive code, binary dumps and tables may raise separate rights questions.

**Action:** retain links, hashes, addresses, symbol names and concise factual descriptions as research evidence. Do not add the original executable, extracted binary resources or large verbatim decompilations to StreetChavesRust without a documented permission basis and a deliberate legal review.

### 4. HD PNGs, converted images and other derivatives

OpenStreetChaves documents `HD/` as a generated set of 3× PNG counterparts to BMP images, using a separate GeraHD tool. It notes that PNGs may also be manually retouched. A generated or manually improved derivative of a copyrighted game image does not automatically become freely reusable.

**Action:** record each derivative's source image, tool/version, transformation settings and author. Keep the output blocked from redistribution until the underlying source asset's rights are resolved.

### 5. SMS/SNES project materials

The SMS project README and PACKAGE document say ROM images are deliberately excluded from its source package because they are copyrighted game data. Its source art is credited to Gerson Almeida's Street Chaves 2 SNES hack. The README says 350 of 404 fighter frames use authentic SNES art, while 54 are stock effect frames. The pose library is captured from the SNES ROM.

The repository's `.bps` files are patches, not the ROM images themselves, and the README/PACKAGE describe required base ROMs and CRC32 identities. A patch or extraction tool does not grant rights to the source ROM or artwork.

**Action:** use the SMS project as a technical reference for ROM tracing, graphics formats and extraction methodology. Do not import its ROM-derived artwork, captured poses, converted fighter frames, title art or palette data into the PC game or Rust project without explicit permission from the relevant rights holders.

### 6. Fonts and vendored dependencies

Observed license evidence:
- OpenStreetChaves README identifies Comic Neue and points to an OFL text file in its font directory. It also warns that Comic Sans MS is a Microsoft font and raises a distribution question for the generated atlas.
- open-street-chaves includes Comic Relief and an `assets/fonts/OFL.txt` file; its README says the font is used under the SIL Open Font License.
- street-chaves-hack-sms includes DejaVu Sans and Montserrat font files. Their licenses must be checked against the exact upstream font releases and the included notices before redistribution.
- The SMS tree vendors Python/Pillow and Z80-related libraries under `pylibs/`; each package retains its own license/notice obligations. The presence of `emu/LICENSE.txt` applies to the emulator component described there, not automatically to all vendored dependencies or tools.

**Action:** maintain a third-party dependency and font register with exact upstream project, version, license identifier, notice file and files used. Do not infer a font license from its filename.

## Reuse decision matrix

| Material | Research/reference use | Copy into Rust repository | Redistribute in a build |
|---|---|---|---|
| Facts about behavior, timings, coordinates and algorithms | Yes, cite exact source/evidence | Yes, as independently written factual documentation | Yes, subject to accuracy and rights review |
| Short source excerpts for analysis | Limited, attributed and necessary | Avoid; prefer links and symbol/file references | No, unless specifically cleared |
| GPL-3.0 source code from either SDL project | Yes | Only after explicit GPL compatibility and attribution decision | Only while satisfying GPL-3.0 and all applicable obligations |
| Original PC sprites, audio, text and stages | Inspect for research where lawful | Blocked pending rights basis | Blocked pending rights basis |
| HD/retouched derivatives of original assets | Inspect for research where lawful | Blocked pending rights basis | Blocked pending rights basis |
| Original PC executable or extracted binary blobs | Analyze locally where lawful | Do not commit | Do not distribute |
| SMS/SNES ROMs and captured artwork | Technical research only | Blocked pending rights basis | Blocked pending rights basis |
| BPS patches | Inspect their format and documented provenance | Separate review required | Separate review required |
| Fonts and third-party libraries | Yes | Only with per-component license/notice review | Only with license compliance |

## Required provenance record for any future imported file

Before a file is copied into StreetChavesRust, record:

1. Exact source repository, URL and commit SHA (or original owner/source).
2. Exact source path and file hash.
3. Author/creator and known upstream origin.
4. Applicable license or written permission, including the specific files it covers.
5. Whether the file is original, extracted, generated, modified or derived.
6. Transformation tool/version and settings, if generated.
7. Attribution/notice obligations and whether redistribution is allowed.
8. Reviewer decision: **allowed**, **allowed with conditions**, **research-only**, or **blocked**.

Unknown must remain unknown. Do not replace missing license evidence with an assumption.

## Current decisions

- **No game assets will be copied into StreetChavesRust in this phase.**
- **No original executable, ROM or extracted binary data will be committed.**
- GPL-3.0 source code reuse is not automatic; prefer independent Rust implementation and record any deliberate reuse with file-level attribution and license review.
- The SMS project remains a supporting technical reference, not a source of PC gameplay behavior or reusable artwork.
- Fonts and vendored dependencies require a component-level license audit before bundling.

## Open legal/provenance questions

- Who owns or controls the rights to each original PC game asset class and the original executable?
- Does OpenStreetChaves have explicit permission to redistribute the checked-in original/extracted assets, and if so, what is its scope?
- Which exact files in the SMS repository are covered by `emu/LICENSE.txt`, and what licenses apply to the rest of the repository?
- What are the exact upstream versions/licenses of all vendored Python/Z80 packages and font files?
- Are any C generated tables or text-index tables derived from protected expressive material rather than factual/functional data?
- What permission, if any, covers the SNES-derived pose library and the converted SMS artwork?

Until answered, the relevant material remains blocked from reuse or redistribution.
