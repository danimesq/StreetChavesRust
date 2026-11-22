# 🌈 Reverse-engineering tools

This directory contains small, reproducible helpers for collecting and validating research inputs. They do not replace the REA MCP service or a disassembler.

## 🌈 REA preflight collector

`rea-preflight.sh` creates a static-analysis evidence bundle for a PE executable.

```bash
bash tools/rea-preflight.sh /path/to/Chaves.exe
# Or choose an explicit output directory:
bash tools/rea-preflight.sh /path/to/Chaves.exe ./analysis/chaves-1.5.2A
```

The script:
- preserves an immutable copy of the input;
- computes and verifies its SHA-256;
- records file identity, host information, tool paths and version banners;
- captures PE headers/imports/sections, ASCII and UTF-16LE strings, and resource listings when the corresponding tools are installed;
- stores stdout reports, stderr logs, command exit statuses, and hashes for generated artifacts;
- never executes the target binary.

### 🌈 Dependencies

Required:
- Bash 4 or newer;
- coreutils utilities used by the script;
- one of `sha256sum`, `shasum`, or `openssl` for SHA-256.

Optional analyzers:
- `file`;
- GNU `objdump` and `strings` (usually from binutils);
- `llvm-readobj`;
- `wrestool` (icoutils);
- `rabin2` (radare2/rizin package, depending on distribution).

Missing optional tools are recorded as skipped. A report with a non-zero exit status is retained with its stderr log and must not be treated as complete.

### 🌈 Output layout

```text
rea-output/<binary>-<sha-prefix>-<utc-time>/
├── README.txt
├── input/       # read-only snapshot
├── reports/     # analyzer output
├── logs/        # run log and stderr
└── manifest/    # input identity, tool versions, exit statuses, hashes
```

The report bundle is tied to the input SHA-256. Do not compare addresses or RVAs across different hashes without revalidating them.

## 🌈 Pascal RNG JSONL validator

The Pascal schema-v2 exporter writes ticks, nested `.rot` invocations,
interpreter branch/loop events, and `SortearValor` draws. Validate its JSONL
and wrapper-local counters with:

```bash
python3 tools/validate-pascal-rng-jsonl.py ./saida/rng.jsonl
python3 -m unittest discover -s tools -p 'test_*.py' -v
```

The validator checks tick and script nesting, source provenance, call ordering,
LongInt/Int64 word costs, replay entries that must not consume MT words, and
positive-range results. It does not make the capture a global MTwist trace:
direct `System.Random` calls outside `SortearValor` remain outside the
captured RNG cursor.

Use the Rust CLI to reconcile a complete trace with the normalizer's static
index and write one JSONL result per draw:

```bash
cargo run --release --bin trace-join -- \
  ./saida/rng.jsonl \
  ./normalized/symbolic-action-tick-index.json \
  ./saida/reconciliation.jsonl
```

The report begins with aggregate counts and then lists every draw's candidate
static sites, symbolic action/tick guards, loop context, and IDs of control
observations from its script invocation. Match statuses are `Unique`,
`Ambiguous`, `Unmatched`, and `NonRotSource`. The full deterministic
scenario is covered by the CI workflow in
[Pascal runtime trace #45040091460](https://github.com/danimesq/StreetChavesRust/actions/runs/45040091460).
The Python validator remains a developer-only helper; the project's Rust-only
policy for tooling is still an open design decision.

See [the Pascal reference trace harness](../docs/pascal-reference-trace-harness.md)
for compile/capture evidence and known limits.

## 🌈 Using the bundle with REA

REA means **Reverse Engineer Anything**. It is the MCP-based reverse-engineering environment, not this shell script. The preflight collector prepares a stable, traceable input package for that workflow.

Suggested sequence:
1. Run the collector and review `manifest/command-status.tsv` and `tool-versions.tsv`.
2. Import `input/Chaves.exe` (or the corresponding snapshot filename) into the REA/Ghidra project.
3. Record the REA project identity, Ghidra version, analyzer configuration, and analysis timestamp.
4. Cross-reference the anchors documented in [the original executable research plan](../docs/original-executable-reverse-engineering.md): `DXTimer1Timer`, `sequenciagolpe`, `defendergolpe`, and `carregaranimacao`.
5. Export address/RVA-linked decompiler notes and call-graph slices. Tie every address to the exact SHA-256.
6. Keep static/decompiler findings separate from controlled runtime observations. Runtime instrumentation belongs in an isolated, documented environment; the preflight script never launches the target.

## 🌈 Research discipline

- REA investigates; the Rust project implements; deterministic tests validate.
- Keep original binary/assets separate from source and build outputs.
- Record origin, version, hash, license/provenance, and analysis environment.
- Do not infer runtime behavior from strings, imports, or decompiler pseudocode alone.
- Do not invent a function name or semantic label when only an address or string reference is known.
