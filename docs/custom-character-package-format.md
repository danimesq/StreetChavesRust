# Custom character package format

This earlier document described a proposed package layout. The executable package validation, native directory adapter, opt-in registry, and browser/WASM bridge are now documented in [Custom character packages](custom-character-packages.md).

The current implementation accepts a package-relative file list containing character.json and the referenced assets. Native hosts can use load_directory; browser hosts supply user-selected file bytes through the WASM exports. The current API validates and registers package data but does not yet connect arbitrary custom fighter definitions to FightSimulation or make the Chapolin sample playable.

The legacy data-root overlay layout described below remains useful as a resource-mapping reference, but it is not the package loader's current on-disk contract.
