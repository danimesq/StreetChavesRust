#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "\${BASH_SOURCE[0]}")/.." && pwd)"
CONFIG="$ROOT/FoundationWebBuild.toml"
GENERATED="$ROOT/.FoundationWebBuild.generated.toml"
FOUNDATION_MANIFEST="\${FOUNDATION_MANIFEST:-$ROOT/../RustMultiConsoleFoundation/Cargo.toml}"

cleanup() { rm -f "$GENERATED"; }
trap cleanup EXIT

python3 - "$CONFIG" "$GENERATED" <<'PY'
import json
import pathlib
import re
import sys

config_path = pathlib.Path(sys.argv[1])
output_path = pathlib.Path(sys.argv[2])
root = config_path.parent
text = config_path.read_text(encoding="utf-8")

# Re-root original assets at the sibling game-data checkout.
text = re.sub(r'^assets-root\s*=\s*"[^"]+"\s*$',
              'assets-root = "."', text, count=1, flags=re.MULTILINE)
text = re.sub(r'^(source\s*=\s*")([^"]+)(")\s*$',
              lambda m: m.group(1) + "../OpenStreetChaves/" + m.group(2) + m.group(3),
              text, flags=re.MULTILINE)

def quote(value):
    return json.dumps(value, ensure_ascii=False)

def slug(value):
    result = re.sub(r"[^a-z0-9_]+", "_", value.lower()).strip("_")
    if not result:
        raise SystemExit(f"cannot create Web resource ID from {value!r}")
    return result

def asset_block(asset_id, source, output, kind):
    return "\n".join([
        "",
        "[[assets]]",
        f"id = {quote(asset_id)}",
        f"source = {quote(source)}",
        f"path = {quote(output)}",
        f"kind = {quote(kind)}",
    ])

packages_root = root / "custom-characters"
package_dirs = sorted(p for p in packages_root.glob("*")
                      if p.is_dir() and (p / "character.json").is_file())
seen_ids = set()
added = 0

for package_dir in package_dirs:
    manifest_path = package_dir / "character.json"
    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    except Exception as error:
        raise SystemExit(f"{manifest_path}: invalid JSON: {error}")

    package_id = manifest.get("id")
    if not isinstance(package_id, str) or not re.fullmatch(r"[a-z0-9_-]{1,64}", package_id):
        raise SystemExit(f"{manifest_path}: invalid package id {package_id!r}")
    if package_id in seen_ids:
        raise SystemExit(f"duplicate custom character package id: {package_id}")
    seen_ids.add(package_id)
    package_slug = slug(package_id)

    manifest_rel = manifest_path.relative_to(root).as_posix()
    text += asset_block(
        f"custom_package_{package_slug}_manifest",
        manifest_rel,
        f"assets/custom-characters/{package_slug}/character.json",
        "data",
    ) + "\n"
    added += 1

    assets = manifest.get("assets")
    if not isinstance(assets, dict):
        raise SystemExit(f"{manifest_path}: assets must be a JSON object")

    for asset_name, relative_path in sorted(assets.items()):
        if not isinstance(asset_name, str) or not isinstance(relative_path, str):
            raise SystemExit(f"{manifest_path}: asset names and paths must be strings")
        if not asset_name or len(asset_name) > 64:
            raise SystemExit(f"{manifest_path}: invalid asset id {asset_name!r}")
        rel = pathlib.PurePosixPath(relative_path)
        if rel.is_absolute() or any(part in ("", ".", "..") for part in rel.parts):
            raise SystemExit(f"{manifest_path}: unsafe package asset path {relative_path!r}")
        source_path = package_dir.joinpath(*rel.parts)
        if not source_path.is_file():
            raise SystemExit(f"{manifest_path}: referenced asset is missing: {relative_path}")

        asset_slug = slug(asset_name)
        source_rel = source_path.relative_to(root).as_posix()
        raw_output = f"assets/custom-characters-raw/{package_slug}/{rel.as_posix()}"
        text += asset_block(
            f"custom_package_{package_slug}_bytes_{asset_slug}",
            source_rel,
            raw_output,
            "data",
        ) + "\n"
        added += 1

        if source_path.suffix.lower() in {".png", ".bmp", ".jpg", ".jpeg", ".webp"}:
            image_output = f"assets/custom-characters/{package_slug}/{rel.as_posix()}"
            text += asset_block(
                f"custom_package_{package_slug}_image_{asset_slug}",
                source_rel,
                image_output,
                "image",
            ) + "\n"
            added += 1

output_path.write_text(text, encoding="utf-8")
print(f"Added {added} resources from {len(package_dirs)} custom character package(s).")
PY

cargo run \
  --manifest-path "$FOUNDATION_MANIFEST" \
  -p foundation-packaging \
  --bin foundation-web-build -- \
  --config "$GENERATED"
