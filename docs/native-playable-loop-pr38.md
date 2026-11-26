# First native runnable loop (PR #38)

## What this adds

- A runnable native binary, \`street-chaves-native\`, backed by the merged Foundation SDL3 host.
- A 400x300 logical scene and the existing authoritative 28 ms simulation clock.
- Two-player input, tick advancement, fighter-frame snapshots, and the Pascal melee consequence resolver when both local sprite heights are known.
- Original local BMP loading for the stage and fighter sprite strips.
- A faithful implementation of the original sprite-sheet separator rule: a column is a separator when its top and bottom pixels are both dark (each RGB component <= 100).
- Source INI \`AlturaSprite\`, \`Velocidade\`, and \`CentroX\` values; player two's sprite-list index offset of 50; white color-key transparency.
- A compact health/charge/power HUD and animated vector fighters for machines without local game data.
- Local Pascal sound-bank loading through Foundation's WAV decoder and shared PCM mixer.

## Run without game assets

    cargo run --features native-foundation --bin street-chaves-native

This opens a procedural training stage with animated placeholder fighters. It is a real input/tick/render loop, but it does not claim to reproduce the original game's art.

## Run with a local OpenStreetChaves folder

    cargo run --features native-foundation --bin street-chaves-native -- \
      --game-root "/path/to/OpenStreetChaves" \
      --stage 1 --p1 1 --p2 4

The loader reads files such as:

- \`Cenarios/Cenario-01.bmp\`
- \`Lutadores/C01.bmp\`, \`Lutadores/C04.bmp\`
- \`Dados/Lutadores/C01.ini\`, \`Dados/Lutadores/C04.ini\`
- \`Dados/Recursos.ini\`, \`Dados/Jogo.ini\`
- Optional WAV banks under \`Sons/\`, \`Musicas/\`, \`Falas/\`, and \`Embutidos/\`

Original game art and audio remain local user-provided files. No extracted sprites or sound files are committed.

## Input

- P1: arrows, Z/X/C/V
- P2: WASD, R/T/Y/U
- Escape: exit

## Accuracy boundary

The stage BMP is rendered as a static background. Fighter sprite strips use the exact original separator-column algorithm and INI pivots. The current simulation executes its implemented Pascal-derived command/frame scripts and can resolve melee contacts once the local sprite heights are loaded.

This is the first runnable loop, not a completed full-game port. The broader original loop still has pending systems such as complete stage object/animation scripts, effect rendering/collision, original game flow and round transitions, AI opponent behavior, and the full physics/knockback sequence. If a required local asset is missing, a diagnostic is recorded and the vector/training fallback remains available.
