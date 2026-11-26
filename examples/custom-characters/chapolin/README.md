# Chapolin Colorado (C16) custom-character example

This folder is a copyable, data-only custom character package for StreetChavesRust.

## What is included

- `character.json`: complete declarative C16 runtime definition.
- 50 sprite-centre offsets from C16.ini.
- 50 body/hurtbox rows from C16.ini.
- 20 ordered attack-box rows from C16.ini.
- Five specials with source-derived durations, sprite-frame timelines, movement windows, invulnerability rules, sounds/effect requests and on-hit recoil.
- Runtime ID 16. The built-in C01-C15 roster identifiers are unchanged.

## Install

Copy this whole directory into the game's user data directory:

```text
<GameData>/custom-characters/chapolin/
```

The native host loads it with `load_directory("custom-characters/chapolin")`. A browser host selects the folder and passes each file's relative path and bytes to the same Rust/WASM package parser. Both then register it with `FightSimulation` and select `chapolin` into a player slot.

Map the host's special-move controls to:

- `special_1`: Corneta Paralisadora
- `special_2`: Marreta Biônica
- `special_3`: Chipote Voador
- `special_4`: Pílulas de Nanicolina
- `special_final`: Sigam-me os Bons!

Use `queue_custom_character_move(slot, move_id, 1)` for weak and `queue_custom_character_move(slot, move_id, 2)` for strong.

## Art and audio

The package intentionally does not include the original C16 bitmap/PNG sprite sheet or audio recordings. Their existence in the Pascal fork does not establish redistribution permission. The manifest has an empty `assets` map; a host can add a cleared sprite under `assets/chapolin.png` and point an asset ID to it.

The headless combat runtime can execute the C16 frame, hitbox and move data without art. A graphical host must supply and register an actual sprite asset to draw the character.

## Provenance and security

C16 is a Chapolin extension in the OpenStreetChaves Pascal fork, not part of the original C01-C15 roster. This example translates its behavior into a bounded JSON schema. It does not include or execute `C16.rot`.

Sources:

- https://github.com/danimesq/OpenStreetChaves/blob/main/Dados/Lutadores/C16.ini
- https://github.com/danimesq/OpenStreetChaves/blob/main/Dados/Lutadores/C16.rot
