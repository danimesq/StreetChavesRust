# Web build with custom characters

Run `scripts/build-web.sh` from any working directory. The script creates a temporary Foundation build manifest, re-roots the original OpenStreetChaves assets at the sibling checkout, and discovers every `custom-characters/*/character.json` and `examples/custom-characters/*/character.json` package in this repository.

Both installed packages and example packages are discovered. Metadata-only example packages are included in the manifest but are not exposed as selectable fighters until a supported sprite image is present. Each package manifest is emitted as a `custom_package_<id>_manifest` data resource. Every declared asset is copied as a byte resource; image files also receive a decoded image resource. The temporary manifest is removed after the build.

Requirements: sibling `../OpenStreetChaves` and `../RustMultiConsoleFoundation` checkouts, Cargo, Python 3, and complete package directories.

The helper rejects duplicate IDs, unsafe relative paths, missing assets and malformed package manifests.
