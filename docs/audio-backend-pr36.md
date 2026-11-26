# Native SDL3 audio backend (PR #36)

## Source contract

The normative source is OpenStreetChaves (Pascal), not the independent C17 port.

- source/game/ChavesSom.pas, source SHA 00b4fabc01505695ed33d251d0afd8fab82fb636.
- source/game/Comum_Recursos.inc, source SHA 21f2e3988c53b12a74235fc35f09f4a202875300.
- source/game/ChavesTipos.pas, source SHA d3c5fd765d458bc4a7d89eb31eefd23b28a4480e.
- source/game/Estado7_Luta.inc, source SHA 940992a9d0371561d6a1c248568049829656afee.
- Dados/Recursos.ini, source SHA eaf667bded6ee65bd13d96feadc6089fbcf5275b.

The Pascal backend is not SDL_mixer. ChavesSom.pas implements its own callback mixer over SDL2. The Rust host uses SDL3's audio callback/stream API while preserving the mixer rules and keeping SDL out of the simulation core.

## Preserved mixer behavior

- 44,100 Hz, signed 16-bit native-endian PCM, stereo.
- WAV assets are decoded by SDL and converted once to the mixer format before playback.
- Volume is applied once to the decoded PCM, using integer percent math and clamping to 0..400.
- 48 simultaneous voices.
- Replaying a sample cuts its previous voice and starts again at frame zero.
- If all voices are occupied, the oldest active voice is stolen.
- Music loops continuously; effects and victory voices do not.
- Voices are summed, then saturated to the signed 16-bit range.
- No audio device is a presentation-host failure, not a simulation failure. The caller can continue running the game silently.

The pure mixer lives in src/audio.rs and has device-free regression tests. The native adapter in src/audio_sdl.rs owns the SDL3 stream, decodes WAV files, converts them with SDL's AudioStream, and manages sample banks. The wasm32 build does not compile or link the SDL adapter.

## Pascal bank indexing

The source's resource loader and sound IDs have intentionally different-looking numbering:

- Effect file Som-01.wav is bank slot 0.
- SomPendente = 10 addresses bank slot 10, which is Som-11.wav.
- SomPendente = 0 means no pending fight sound.
- The sequential effects loader stops at the first missing WAV and reserves one final empty slot when at least one effect exists.
- The looped music bank reserves slot 0 for Embutidos/Musica.wav. Selecting fight music replaces that slot; loading and starting the track are separate operations.
- Victory voice order is fighter-first, then variant. The source computes its index as (Personagem - 1) * 2 + K.

SdlAudioOutput::play_pending_fight_sound mirrors L7_CenarioECamera: call it once immediately after the fight simulation tick, not once per render/presentation frame. If the pending index is outside the loaded bank, it remains pending just as the Pascal count guard leaves it untouched. Once the bank slot exists, the host consumes the request even if that particular WAV is empty or failed to decode.

The custom C16 Chapolin package's IDs 45..49 use this same effect bank. They resolve to Som-46.wav through Som-50.wav when those local assets exist.

## Resource folders and volume overrides

SdlAudioOutput::load_pascal_resources receives the game root directory and reads:

- Dados/Recursos.ini for the effect, music and victory voice patterns.
- Dados/Som.ini for default volume overrides.
- Chaves.ini for per-user overrides, applied after Dados/Som.ini.

For each file, volume lookup priority is file stem (for example Som-05), parent folder (Sons, Musicas or Falas), then Padrao, falling back to 100. Values are clamped to 0..400. Resource paths are loaded relative to the supplied game root; absolute paths remain absolute.

Missing effect files terminate the sequential effect list. Missing voice files reserve their configured numbered slot. The loader records decode failures in PascalAudioLayout::warnings rather than shifting IDs. The repository does not include extracted game WAVs; users point the host at their own local game-data directory.

## Host integration order

After one authoritative FightSimulation tick:

1. Read input and advance simulation.
2. Call SdlAudioOutput::play_pending_fight_sound(&layout, &mut fight).
3. Build/render the presentation snapshot.
4. Present the frame.

The host may call play_victory_voice for the Pascal-computed victory index and use load_fight_music followed by play_music when the source's music-pending flag is consumed. Do not poll pending sound while rendering; it is a one-slot event, not a persistent sound state.

## Remaining work

This PR provides actual native WAV playback/mixing and Pascal resource-bank loading. It does not yet add the game window, renderer, sprite extraction/atlas mapping, keyboard/gamepad host loop, browser WebAudio, or audio-device hotplug recovery. Those belong to later presentation-host slices.
