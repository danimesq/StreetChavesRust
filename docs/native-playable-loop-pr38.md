# Native playable loop (PR #41)

## Startup screens

The native executable now starts at boot and moves through:

1. **Opening intro**: a short original vector sequence with a vila-to-logo rhythm.
2. **Title / Start**: title screen with Start prompt.
3. **Fighter selection**: 15-character roster, separate P1/P2 cursors and confirmation.
4. **Fight**: enters the 28 ms game simulation only after the selection step.

P1 navigates with arrows and confirms with Z. P2 can join with R from the title screen or with WASD/R on the selection screen. When P2 does not join, the second selected fighter remains a stationary training opponent. It is not CPU AI.

## Run without local game assets

\`\`\`sh
cargo run --features native-foundation --bin street-chaves-native
\`\`\`

This uses the generated opening/menu artwork, a procedural training stage and animated vector fighters.

## Run with a local OpenStreetChaves directory

\`\`\`sh
cargo run --features native-foundation --bin street-chaves-native -- \\
  --game-root "/path/to/OpenStreetChaves" --stage 1 --p1 1 --p2 4
\`\`\`

The executable reads local Pascal files such as:

- \`Cenarios/Cenario-01.bmp\`
- \`Lutadores/C01.bmp\` and \`Lutadores/C04.bmp\`
- \`Dados/Recursos.ini\`
- \`Dados/Lutadores/C01.ini\`, \`C04.ini\`
- \`Dados/Jogo.ini\`
- Optional WAV banks in \`Sons/\`, \`Musicas/\`, \`Falas/\` and \`Embutidos/\`

The sprite loader implements the original dark-column separator test (top and bottom pixels RGB <= 100), reads \`AlturaSprite\`, \`Velocidade\` and \`CentroX\`, applies the pure-white color key and preserves the second player's +50 sprite-list displacement. No original proprietary assets are checked into this repository.

## Architecture

Foundation owns SDL3, its window, input events, texture registration, WAV decoding and PCM mixer. StreetChavesRust owns the startup state machine, local resource parsing, 28 ms clock, combat simulation, presentation and HUD.

## Current fidelity boundary

The opening art is a new vector scene, not a pixel-identical reconstruction of the original Pascal intro. The title and selection screens are a first application flow, not the entire original menu system. Options/key-binding screens, attract demo, full game flow, stage animations and destructible objects, effect renderer/collision, round transitions, records, continue/final screens, complete physics/knockback and CPU AI remain future work.
