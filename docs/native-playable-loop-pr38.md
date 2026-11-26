# First native playable loop (PR #38)

## Startup flow

The executable now begins at the opening sequence instead of jumping directly into a fight:

1. **Opening intro**: short, animated vector sequence with a vila-to-logo rhythm.
2. **Title / Start screen**: the Street Chaves title and a Start prompt.
3. **Fighter selection**: a 15-character grid with independent P1/P2 cursors and confirmation.
4. **Fight**: enters the 28 ms simulation after selection.

P1 navigates with the arrow keys and confirms with Z. P2 can join from the title screen with R or during selection with WASD, then confirms with R. If P2 does not join, the second selected fighter remains a stationary training opponent. There is no CPU opponent AI yet.

## Run without original assets

```sh
cargo run --features native-foundation --bin street-chaves-native
```

The opening, title, and character-selection screens use generated vector/pixel-art presentation. The fight uses a procedural training stage and animated placeholder fighters.

## Run with a local OpenStreetChaves folder

```sh
cargo run --features native-foundation --bin street-chaves-native -- \
  --game-root "/path/to/OpenStreetChaves" \
  --stage 1 --p1 1 --p2 4
```

The IDs passed through `--p1` and `--p2` set the initial cursors in the selection screen. Local original assets load after confirmation:

- `Cenarios/Cenario-01.bmp`
- `Lutadores/C01.bmp`, `Lutadores/C04.bmp`
- `Dados/Lutadores/C01.ini`, `Dados/Lutadores/C04.ini`
- `Dados/Recursos.ini`, `Dados/Jogo.ini`
- Optional WAV banks under `Sons/`, `Musicas/`, `Falas/`, and `Embutidos/`

The sprite-sheet loader reproduces the Pascal separator-column rule: top and bottom pixels both need RGB components <= 100. It applies the source white color key, reads `AlturaSprite`, `Velocidade`, and `CentroX`, and uses the P2 +50 sprite-list displacement. No proprietary game BMP/WAV asset is committed.

## Input

- P1: arrows to move/select, Z to confirm, X/C/V for other fight buttons.
- P2: WASD to move/select, R to join/confirm, T/Y/U for other fight buttons.
- Escape: quit.

The Foundation host owns SDL3, the native window, texture uploads, WAV decoding and the PCM mixer. StreetChavesRust owns the opening/menu state machine, local resource interpretation, 28 ms clock, game simulation and presentation.

## Accuracy boundary

The menu flow and vector opening are new Rust presentation, not a pixel-perfect port of the original intro assets. The actual Pascal title/attract animation assets still need separate provenance-aware loading. In the fight, implemented Pascal-derived scripts run and melee contacts are resolved once local fighter heights are known. HUD life and charge ratios follow Pascal's 125 life and 102 charge maxima.

Still pending beyond this first startup-to-fight loop: the original intro/attract animation, full character-select artwork/layout, options/key-binding screens, configuration persistence, stage animation/object scripts, effect rendering and collision, round transitions/records/continue/final screens, AI opponent behavior, and the complete physics/knockback sequence.
