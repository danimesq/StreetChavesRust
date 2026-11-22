#!/usr/bin/env bash
# 🌈 REA preflight: create a reproducible static-analysis evidence bundle.
# 🌈 This script does not replace REA, Ghidra, or runtime instrumentation.
# 🌈 It preserves the input, records hashes/tool versions, and captures reports.

set -Eeuo pipefail
IFS=$'\n\t'
umask 077

INPUT=${1:-}
REQUESTED_OUT=${2:-}

usage() {
  cat <<'USAGE'
Usage:
  rea-preflight.sh <pe-binary> [output-directory]

Creates a timestamped evidence bundle containing:
  input/       read-only snapshot of the analyzed binary
  reports/     PE headers, imports, sections, strings, resource listings
  logs/        command logs and errors
  manifest/    hashes, file metadata, tool versions, run metadata

Required:
  Bash 4+, coreutils, and one SHA-256 implementation
Optional:
  file, objdump, strings, llvm-readobj, wrestool, rabin2

The input is never modified. The script does not execute the target binary.
USAGE
}

die() {
  printf 'ERROR: %s\n' "$*" >&2
  exit 2
}

if [[ -z "$INPUT" || "$INPUT" == "-h" || "$INPUT" == "--help" ]]; then
  usage
  [[ -n "$INPUT" ]] && exit 0
  exit 2
fi

[[ -f "$INPUT" ]] || die "Input file not found: $INPUT"
[[ -r "$INPUT" ]] || die "Input file is not readable: $INPUT"

# 🌈 Resolve the input path without depending on GNU realpath.
INPUT_DIR=$(cd -- "$(dirname -- "$INPUT")" && pwd -P)
INPUT="$INPUT_DIR/$(basename -- "$INPUT")"
BASENAME=$(basename -- "$INPUT")
STAMP=$(date -u '+%Y%m%dT%H%M%SZ')

sha256_file() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum -- "$1" | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 -- "$1" | awk '{print $1}'
  elif command -v openssl >/dev/null 2>&1; then
    openssl dgst -sha256 "$1" | awk '{print $NF}'
  else
    die "Need sha256sum, shasum, or openssl for SHA-256."
  fi
}

INPUT_SHA=$(sha256_file "$INPUT")
SHORT_SHA=${INPUT_SHA:0:12}

if [[ -n "$REQUESTED_OUT" ]]; then
  OUT="$REQUESTED_OUT"
else
  OUT="rea-output/${BASENAME%.*}-${SHORT_SHA}-$STAMP"
fi

mkdir -p -- "$OUT"/{input,reports,logs,manifest}
OUT=$(cd -- "$OUT" && pwd -P)
SNAPSHOT="$OUT/input/$BASENAME"

# 🌈 Preserve the exact input bytes in the bundle and make the copy read-only.
cp -- "$INPUT" "$SNAPSHOT"
chmod a-w "$SNAPSHOT"
SNAPSHOT_SHA=$(sha256_file "$SNAPSHOT")
[[ "$SNAPSHOT_SHA" == "$INPUT_SHA" ]] || die "Snapshot hash mismatch; refusing to continue."

RUN_LOG="$OUT/logs/run.log"
: > "$RUN_LOG"
exec > >(tee -a "$RUN_LOG") 2>&1

printf 'REA preflight started: %s\n' "$STAMP"
printf 'Input: %s\n' "$INPUT"
printf 'SHA-256: %s\n' "$INPUT_SHA"
printf 'Output: %s\n' "$OUT"

# 🌈 Run optional analyzers without making missing tools fatal.
# 🌈 Every command gets a stdout report and a separate stderr/exit log.
run_report() {
  local name="$1"
  shift
  local stdout="$OUT/reports/$name.txt"
  local stderr="$OUT/logs/$name.stderr.log"
  local status=0

  printf '\n[RUN] %s' "$name"
  printf ' %q' "$@"
  printf '\n'

  if "$@" >"$stdout" 2>"$stderr"; then
    status=0
  else
    status=$?
    printf '[WARN] %s exited with status %s; inspect %s\n' "$name" "$status" "$stderr"
  fi
  printf '%s\t%s\n' "$name" "$status" >> "$OUT/manifest/command-status.tsv"
}

# 🌈 Tool versions: record exact executable paths and version banners.
{
  printf 'tool\tpath\tversion\n'
  for tool in bash sha256sum shasum openssl file objdump strings llvm-readobj wrestool rabin2; do
    if command -v "$tool" >/dev/null 2>&1; then
      path=$(command -v "$tool")
      version=$("$tool" --version 2>&1 | head -n 1 || true)
      [[ -n "$version" ]] || version="version banner unavailable"
      printf '%s\t%s\t%s\n' "$tool" "$path" "${version//$'\t'/ }"
    else
      printf '%s\t%s\t%s\n' "$tool" "not-installed" "-"
    fi
  done
} > "$OUT/manifest/tool-versions.tsv"

# 🌈 Basic file identity and a stable input manifest.
{
  printf 'field\tvalue\n'
  printf 'original_path\t%s\n' "$INPUT"
  printf 'snapshot_path\t%s\n' "$SNAPSHOT"
  printf 'basename\t%s\n' "$BASENAME"
  printf 'size_bytes\t%s\n' "$(wc -c < "$SNAPSHOT" | tr -d ' ')"
  printf 'sha256\t%s\n' "$INPUT_SHA"
  printf 'captured_utc\t%s\n' "$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
  printf 'host\t%s\n' "$(uname -a)"
} > "$OUT/manifest/input.tsv"

if command -v file >/dev/null 2>&1; then
  run_report file-identification file -k "$SNAPSHOT"
else
  printf '[SKIP] file: not installed\n'
fi

if command -v objdump >/dev/null 2>&1; then
  run_report pe-objdump-x objdump -x "$SNAPSHOT"
  run_report pe-objdump-p objdump -p "$SNAPSHOT"
else
  printf '[SKIP] objdump: not installed\n'
fi

if command -v strings >/dev/null 2>&1; then
  run_report strings-ascii strings -a -t x -n 4 "$SNAPSHOT"
  run_report strings-utf16le strings -a -e l -t x -n 4 "$SNAPSHOT"
else
  printf '[SKIP] strings: not installed\n'
fi

if command -v llvm-readobj >/dev/null 2>&1; then
  run_report pe-llvm-readobj llvm-readobj --file-headers --sections --coff-imports --coff-exports --coff-resources "$SNAPSHOT"
else
  printf '[SKIP] llvm-readobj: not installed\n'
fi

if command -v wrestool >/dev/null 2>&1; then
  run_report pe-resource-list wrestool --list "$SNAPSHOT"
else
  printf '[SKIP] wrestool: not installed\n'
fi

if command -v rabin2 >/dev/null 2>&1; then
  run_report pe-rabin-info rabin2 -I "$SNAPSHOT"
  run_report pe-rabin-imports rabin2 -i "$SNAPSHOT"
  run_report pe-rabin-sections rabin2 -S "$SNAPSHOT"
  run_report pe-rabin-resources rabin2 -R "$SNAPSHOT"
  run_report pe-rabin-strings rabin2 -zz "$SNAPSHOT"
else
  printf '[SKIP] rabin2: not installed\n'
fi

cat > "$OUT/README.txt" <<'BUNDLE'
REA preflight evidence bundle

This bundle was generated without executing the target binary.
- input/: immutable copy of the analyzed file
- reports/: static PE, import, section, string, and resource reports
- logs/: run log and stderr/exit logs for each tool
- manifest/: input identity, tool versions, command status, artifact hashes

Review command-status.tsv and tool-versions.tsv before interpreting missing reports.
A report generated by one tool is not independent confirmation of another report.
Addresses and RVAs must always be tied to the input SHA-256 in input.tsv.

Next REA workflow:
1. Import the immutable input snapshot into the selected REA/Ghidra project.
2. Record the analysis project/tool versions and applied analyzers.
3. Cross-reference the documented anchors and export address-linked findings.
4. Keep static/decompiler observations separate from controlled runtime observations.
BUNDLE

# 🌈 A machine-readable hash list for every generated artifact (excluding itself).
(
  cd -- "$OUT"
  find input reports logs manifest -type f ! -name 'artifact-sha256.tsv' -print0; printf '%s\\0' README.txt |
    sort -z |
    while IFS= read -r -d '' path; do
      printf '%s\t%s\n' "$(sha256_file "$path")" "$path"
    done
) > "$OUT/manifest/artifact-sha256.tsv"


printf '\n[DONE] Evidence bundle: %s\n' "$OUT"
printf '[DONE] Input SHA-256: %s\n' "$INPUT_SHA"
printf '[NOTE] The target was not executed. Review warnings and skipped tools.\n'
