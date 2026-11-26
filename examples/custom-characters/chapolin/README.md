# Chapolin custom-character package example

This folder is a package-manifest example for the custom-character system. It contains no compiled executable and intentionally does not redistribute the Chapolin sprite/audio assets.

## Contents

- character.json: stable package ID, observed C16 attributes, and the five named special moves.
- assets/: add cleared artwork here when you have the rights to distribute it.

## Native host

Copy this entire folder to the game's user data directory:

```text
<GameData>/custom-characters/chapolin/
```

The native host can discover it with the custom-character directory loader, install it in the registry, and let the player enable/disable it. The package manager does not require rebuilding the executable.

## Web host

Select this folder through the browser's package-folder picker. The host passes each relative file path and its bytes to the WASM package bridge. The same manifest and validation rules are used; package files are not compiled into the WASM module.

## Current limitation

This example proves package identity, metadata, and cross-platform package ingestion/activation. It is not yet a playable Chapolin fighter: the current simulation still has fixed built-in fighter state and does not yet interpret arbitrary custom fighter moves, animations, collision boxes, AI, and effect behavior. Add the cleared fighter artwork and complete the runtime integration before presenting this package as playable.
