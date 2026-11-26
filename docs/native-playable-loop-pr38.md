# Native playable loop (PR #41)

## Startup screens

The native executable now starts at boot and moves through:

1. **Opening intro**: a short original vector sequence with a vila-to-logo rhythm.
2. **Title / Arcade cabinet**: insert credits with 5, then press 1 for P1 or 2 for P2.
3. **F1 configuration**: edit twelve original settings with up/down and left/right; F1 saves and returns.
4. **F2 remapping**: capture eight P1 physical keys; duplicate keys are rejected; F2 exits.
5. **Fighter selection**: 15-character roster, separate P1/P2 cursors and confirmation.
6. **Fight**: enters the 28 ms game simulation. If P2 has not joined, Common.rot AI controls the second fighter. P2 can spend another credit with 2 to join a live fight and take over the slot.

P1 navigates with arrows and confirms with its mapped weak-punch key (Z by default). P2 uses WASD and R/T/Y/U (R confirms by default). The P1 map is shared between SDL3 and the browser host through Foundation's physical-key contract. Native preferences are stored in StreetChavesRust.settings.json; browser preferences use localStorage.

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

## Original state and fidelity boundary

The original project separates the flow into `CHS_ABERTURA`, `CHS_CONFIG`, `CHS_TECLAS`, `CHS_SELECAO`, `CHS_ADVERSARIOS`, `CHS_PREPARALUTA` and `CHS_LUTA`. StreetChavesRust now has a first credit ledger, editable/persisted F1 settings, shared P1 F2 remapping, and Common.rot CPU behavior for an unjoined P2 slot. The current CPU probability curve is provisional and per-character AI is not fully connected. The round manager, several F1 setting consumers, attract mode, campaign order, full combat fidelity and end-of-fight states remain incomplete. The authoritative per-screen matrix and source links live in [CHS-SCREEN-STATUS.md](CHS-SCREEN-STATUS.md); detailed settings and boundaries are in [arcade-credits-settings-and-ai.md](arcade-credits-settings-and-ai.md).

## Current fidelity boundary

The opening art is a new vector scene, not a pixel-identical reconstruction of the original Pascal intro. The title and selection screens are still a first application flow. F1 persists all twelve values, but round time, rounds, blood, language, sound/music preview, flash and complete HUD behavior still await their consuming systems. The CPU uses the common ROT AI path, not the full per-character AI set. Attract demo, campaign opponent order, stage animations and destructible objects, effect rendering/collision, round transitions, records, continue/final screens and complete physics/knockback remain future work.
