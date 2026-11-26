//! Native SDL3 adapter for the Pascal-shaped Street Chaves audio mixer.
//!
//! This is a presentation-host module. The combat core and the PCM mixer remain usable
//! without SDL; only native builds include this adapter.

use crate::audio::{AudioMixer, SampleId, MIXER_CHANNELS, MIXER_RATE_HZ};
use crate::fight_simulation::FightSimulation;
use sdl3::audio::{
    AudioCallback, AudioFormat, AudioSpec, AudioSpecWAV, AudioStream, AudioStreamWithCallback,
};
use sdl3::{AudioSubsystem, Sdl};
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const DEFAULT_EFFECT_PATTERN: &str = "Sons/Som-##.wav";
const DEFAULT_MUSIC_PATTERN: &str = "Musicas/Musica-##.wav";
const DEFAULT_MENU_MUSIC: &str = "Embutidos/Musica.wav";
const DEFAULT_VICTORY_VARIANTS: &str = "Falas/Fala-##-a.wav,Falas/Fala-##-b.wav";

impl AudioCallback<i16> for AudioMixer {
    fn callback(&mut self, stream: &mut AudioStream, requested: i32) {
        let samples = self.render_callback_buffer(requested.max(0) as usize);
        // SDL pulls data on its audio thread. Keep this callback allocation-free after warm-up.
        let _ = stream.put_data_i16(samples);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioBankId(usize);

#[derive(Debug)]
struct AudioBank {
    sample_ids: Vec<usize>,
    looped: bool,
}

#[derive(Debug)]
pub enum AudioHostError {
    Sdl(String),
    InvalidBank,
}

impl fmt::Display for AudioHostError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sdl(message) => write!(formatter, "SDL audio error: {message}"),
            Self::InvalidBank => formatter.write_str("invalid audio bank identifier"),
        }
    }
}

impl std::error::Error for AudioHostError {}

/// The exact resource-bank layout used by OpenStreetChaves' Pascal resource loader.
///
/// Effects are zero-based, so sound ID 10 addresses slot 10 (Sons/Som-11.wav).
/// Music slot 0 starts as the embedded menu track and is replaced by each selected fight track.
/// Victory voices are appended fighter-first, then variant, matching the Pascal load order.
#[derive(Debug, Clone)]
pub struct PascalAudioLayout {
    pub effects: AudioBankId,
    pub music: AudioBankId,
    pub victory: AudioBankId,
    pub fight_music_count: usize,
    root: PathBuf,
    fight_music_pattern: String,
    volumes: IniDocument,
    pub warnings: Vec<String>,
}

/// Live native audio output. SDL stream is declared first so it is dropped before SDL subsystems.
pub struct SdlAudioOutput {
    stream: AudioStreamWithCallback<AudioMixer>,
    _audio: AudioSubsystem,
    _context: Sdl,
    banks: Vec<AudioBank>,
    samples: Vec<Option<Arc<[i16]>>>,
}

impl SdlAudioOutput {
    /// Open the SDL3 playback stream in Pascal's canonical 44.1 kHz S16 stereo format.
    ///
    /// The game host should treat an error here as "audio unavailable" and keep the
    /// simulation running silently, matching the original Pascal startup behavior.
    pub fn open() -> Result<Self, AudioHostError> {
        let context = sdl3::init().map_err(|error| AudioHostError::Sdl(error.to_string()))?;
        let audio = context
            .audio()
            .map_err(|error| AudioHostError::Sdl(error.to_string()))?;
        let output_spec = AudioSpec::new(
            Some(MIXER_RATE_HZ as i32),
            Some(MIXER_CHANNELS as i32),
            Some(AudioFormat::s16_sys()),
        );
        let mut stream = audio
            .open_playback_stream(&output_spec, AudioMixer::default())
            .map_err(|error| AudioHostError::Sdl(error.to_string()))?;
        stream
            .resume()
            .map_err(|error| AudioHostError::Sdl(error.to_string()))?;

        Ok(Self {
            stream,
            _audio: audio,
            _context: context,
            banks: Vec::new(),
            samples: Vec::new(),
        })
    }

    /// Create a bank. Music banks loop; effects and victory voices normally do not.
    pub fn create_bank(&mut self, looped: bool) -> AudioBankId {
        let id = AudioBankId(self.banks.len());
        self.banks.push(AudioBank {
            sample_ids: Vec::new(),
            looped,
        });
        id
    }

    /// Reserve an empty numbered slot, preserving the Pascal bank indexing.
    pub fn add_empty_slot(&mut self, bank: AudioBankId) -> Result<usize, AudioHostError> {
        let bank_record = self.banks.get(bank.0).ok_or(AudioHostError::InvalidBank)?;
        let index = bank_record.sample_ids.len();
        let sample_id = self.samples.len();
        self.samples.push(None);
        self.banks[bank.0].sample_ids.push(sample_id);
        Ok(index)
    }

    /// Append a WAV slot. Missing files reserve an empty slot; decode errors are returned
    /// after reserving it, so a bad asset never shifts later sound IDs.
    pub fn add_wav(
        &mut self,
        bank: AudioBankId,
        path: impl AsRef<Path>,
        volume_percent: i32,
    ) -> Result<usize, AudioHostError> {
        let bank_record = self.banks.get(bank.0).ok_or(AudioHostError::InvalidBank)?;
        let index = bank_record.sample_ids.len();
        let sample_id = self.samples.len();
        self.samples.push(None);
        self.banks[bank.0].sample_ids.push(sample_id);

        let path = path.as_ref();
        if path.is_file() {
            let pcm = self.decode_wav(path, volume_percent)?;
            self.samples[sample_id] = Some(pcm);
        }
        Ok(index)
    }

    /// Replace a numbered bank slot, stopping its current voice before releasing old PCM.
    pub fn replace_wav(
        &mut self,
        bank: AudioBankId,
        index: usize,
        path: impl AsRef<Path>,
        volume_percent: i32,
    ) -> Result<bool, AudioHostError> {
        let sample_id = match self
            .banks
            .get(bank.0)
            .and_then(|record| record.sample_ids.get(index))
            .copied()
        {
            Some(sample_id) => sample_id,
            None => return Ok(false),
        };

        self.stop_sample(SampleId(sample_id));
        self.samples[sample_id] = None;

        let path = path.as_ref();
        if path.is_file() {
            let pcm = self.decode_wav(path, volume_percent)?;
            self.samples[sample_id] = Some(pcm);
        }
        Ok(true)
    }

    /// Play one sample by zero-based bank index.
    pub fn play(&mut self, bank: AudioBankId, index: usize) -> bool {
        let (sample_id, looped) = match self.banks.get(bank.0) {
            Some(record) => match record.sample_ids.get(index) {
                Some(sample_id) => (*sample_id, record.looped),
                None => return false,
            },
            None => return false,
        };
        let pcm = match self.samples.get(sample_id).and_then(|sample| sample.clone()) {
            Some(pcm) => pcm,
            None => return false,
        };

        match self.stream.lock() {
            Some(mut callback) => callback.play(SampleId(sample_id), pcm, looped),
            None => false,
        }
    }

    /// Stop the currently playing voice(s) of one numbered sample.
    pub fn stop(&mut self, bank: AudioBankId, index: usize) -> bool {
        let sample_id = match self
            .banks
            .get(bank.0)
            .and_then(|record| record.sample_ids.get(index))
            .copied()
        {
            Some(sample_id) => sample_id,
            None => return false,
        };
        self.stop_sample(SampleId(sample_id));
        true
    }

    /// Number of reserved samples in a bank, including empty/missing files.
    pub fn bank_len(&self, bank: AudioBankId) -> Option<usize> {
        self.banks.get(bank.0).map(|record| record.sample_ids.len())
    }

    /// Load the folders and volume rules described by Dados/Recursos.ini and Dados/Som.ini.
    ///
    /// Chaves.ini overrides Dados/Som.ini. Files are loaded locally from game_root; extracted
    /// original audio is not copied into or redistributed by this repository.
    pub fn load_pascal_resources(
        &mut self,
        game_root: impl AsRef<Path>,
    ) -> PascalAudioLayout {
        let root = game_root.as_ref().to_path_buf();
        let resources = IniDocument::read(&root.join("Dados/Recursos.ini"));
        let mut volumes = IniDocument::read(&root.join("Dados/Som.ini"));
        volumes.merge(IniDocument::read(&root.join("Chaves.ini")));

        let effects_pattern = resources
            .get("Sons", "Arquivo")
            .unwrap_or(DEFAULT_EFFECT_PATTERN)
            .to_owned();
        let music_pattern = resources
            .get("Musicas", "Arquivo")
            .unwrap_or(DEFAULT_MUSIC_PATTERN)
            .to_owned();
        let menu_music = resources
            .get("Musicas", "Menu")
            .unwrap_or(DEFAULT_MENU_MUSIC)
            .to_owned();
        let victory_variants = resources
            .get("Falas", "Variantes")
            .unwrap_or(DEFAULT_VICTORY_VARIANTS)
            .split(',')
            .map(str::trim)
            .filter(|pattern| !pattern.is_empty())
            .map(str::to_owned)
            .collect::<Vec<_>>();

        let mut warnings = Vec::new();

        // Pascal adds effects sequentially and stops at the first missing numbered WAV.
        let effects = self.create_bank(false);
        for number in 1..=255 {
            let relative = numbered_file(&effects_pattern, number);
            let path = rooted_path(&root, &relative);
            if !path.is_file() {
                break;
            }
            let volume = volume_for_path(&path, &volumes);
            if let Err(error) = self.add_wav(effects, &path, volume) {
                warnings.push(format!("{}: {error}", path.display()));
            }
        }
        // The Pascal loader appends one final empty sample after the first missing effect file.
        if self.bank_len(effects).unwrap_or(0) > 0 {
            let _ = self.add_empty_slot(effects);
        }

        // The original music bank is looped and reserves slot zero for the menu track.
        let music = self.create_bank(true);
        let menu_path = rooted_path(&root, &menu_music);
        if let Err(error) = self.add_wav(music, &menu_path, volume_for_path(&menu_path, &volumes)) {
            warnings.push(format!("{}: {error}", menu_path.display()));
        }

        // Fight tracks are counted here, then loaded into the existing slot zero on selection.
        let mut fight_music_count = 0;
        for number in 1..=255 {
            let path = rooted_path(&root, &numbered_file(&music_pattern, number));
            if !path.is_file() {
                break;
            }
            fight_music_count += 1;
        }

        // Voices are appended fighter-first, then in the configured variant order.
        let victory = self.create_bank(false);
        let fighter_data_pattern = resources
            .get("Lutadores", "Dados")
            .unwrap_or("Dados/Lutadores/C##.ini");
        let configured_fighters = resources
            .get("Falas", "Quantidade")
            .and_then(|value| value.parse::<usize>().ok())
            .or_else(|| {
                resources
                    .get("Lutadores", "Quantidade")
                    .and_then(|value| value.parse::<usize>().ok())
            });
        // Pascal counts FQtdLutadores from the [Lutadores] Dados pattern, not from WAV
        // presence. Missing voice files still reserve each fighter/variant bank position.
        let fighter_count = configured_fighters
            .unwrap_or_else(|| count_numbered_files(&root, fighter_data_pattern));
        for fighter in 1..=fighter_count {
            for pattern in &victory_variants {
                let path = rooted_path(&root, &numbered_file(pattern, fighter));
                let volume = volume_for_path(&path, &volumes);
                if let Err(error) = self.add_wav(victory, &path, volume) {
                    warnings.push(format!("{}: {error}", path.display()));
                }
            }
        }

        PascalAudioLayout {
            effects,
            music,
            victory,
            fight_music_count,
            root,
            fight_music_pattern: music_pattern,
            volumes,
            warnings,
        }
    }

    /// Load a selected fight track into music slot zero, preserving the original bank design.
    /// Loading does not automatically start playback; the caller consumes the Pascal music
    /// pending flag and then calls play(layout.music, 0).
    pub fn load_fight_music(
        &mut self,
        layout: &PascalAudioLayout,
        number: usize,
    ) -> Result<bool, AudioHostError> {
        if number == 0 || number > layout.fight_music_count {
            return Ok(false);
        }
        let relative = numbered_file(&layout.fight_music_pattern, number);
        let path = rooted_path(&layout.root, &relative);
        self.replace_wav(
            layout.music,
            0,
            &path,
            volume_for_path(&path, &layout.volumes),
        )
    }

    /// Consume and play the one-slot Pascal SomPendente request after each simulation tick.
    ///
    /// The ID is the bank index, not a one-based filename. The event is cleared once its
    /// numbered slot exists, even if its WAV failed to decode or the device cannot play it.
    pub fn play_pending_fight_sound(
        &mut self,
        layout: &PascalAudioLayout,
        fight: &mut FightSimulation,
    ) -> bool {
        let sound_id = match fight.state.pending_sound {
            Some(sound_id) => usize::from(sound_id),
            None => return false,
        };
        if self
            .bank_len(layout.effects)
            .map(|count| sound_id >= count)
            .unwrap_or(true)
        {
            return false;
        }

        let _ = fight.take_pending_sound();
        let _ = self.play(layout.effects, sound_id);
        true
    }

    /// Play a victory phrase by its Pascal index: (fighter_id - 1) * variants + variant.
    pub fn play_victory_voice(&mut self, layout: &PascalAudioLayout, index: usize) -> bool {
        self.play(layout.victory, index)
    }

    /// Play the current menu/fight music from bank slot zero.
    pub fn play_music(&mut self, layout: &PascalAudioLayout) -> bool {
        self.play(layout.music, 0)
    }

    pub fn stop_music(&mut self, layout: &PascalAudioLayout) -> bool {
        self.stop(layout.music, 0)
    }

    fn stop_sample(&mut self, sample_id: SampleId) {
        if let Some(mut callback) = self.stream.lock() {
            callback.stop(sample_id);
        }
    }

    fn decode_wav(
        &self,
        path: &Path,
        volume_percent: i32,
    ) -> Result<Arc<[i16]>, AudioHostError> {
        let wav = AudioSpecWAV::load_wav(path)
            .map_err(|error| AudioHostError::Sdl(error.to_string()))?;
        let source_spec = AudioSpec::new(
            Some(wav.freq),
            Some(i32::from(wav.channels)),
            Some(wav.format),
        );
        let mixer_spec = AudioSpec::new(
            Some(MIXER_RATE_HZ as i32),
            Some(MIXER_CHANNELS as i32),
            Some(AudioFormat::s16_sys()),
        );
        let mut converter = self
            ._audio
            .new_stream(Some(&source_spec), Some(&mixer_spec))
            .map_err(|error| AudioHostError::Sdl(error.to_string()))?;
        converter
            .put_data(wav.buffer())
            .map_err(|error| AudioHostError::Sdl(error.to_string()))?;
        converter
            .flush()
            .map_err(|error| AudioHostError::Sdl(error.to_string()))?;

        let available = converter
            .available_bytes()
            .map_err(|error| AudioHostError::Sdl(error.to_string()))?
            .max(0) as usize;
        let mut pcm = vec![0_i16; available / std::mem::size_of::<i16>()];
        let read = converter
            .read_i16_samples(&mut pcm)
            .map_err(|error| AudioHostError::Sdl(error.to_string()))?;
        pcm.truncate(read - (read % MIXER_CHANNELS));
        apply_volume(&mut pcm, volume_percent);
        Ok(Arc::from(pcm))
    }
}

/// Count a numbered resource series exactly until the first missing path.
fn count_numbered_files(root: &Path, pattern: &str) -> usize {
    (1..=255)
        .take_while(|number| rooted_path(root, &numbered_file(pattern, *number)).is_file())
        .count()
}

fn rooted_path(root: &Path, resource: &str) -> PathBuf {
    let path = Path::new(resource);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

/// Match Pascal ArquivoNN: replace only the first consecutive run of '#' characters.
fn numbered_file(pattern: &str, number: usize) -> String {
    let Some(start) = pattern.find('#') else {
        return pattern.to_owned();
    };
    let width = pattern[start..]
        .chars()
        .take_while(|character| *character == '#')
        .count();
    let end = start + width;
    let number = format!("{number:0width$}");
    format!("{}{}{}", &pattern[..start], number, &pattern[end..])
}

/// Apply the source's one-time volume scaling after conversion to S16 stereo PCM.
fn apply_volume(samples: &mut [i16], volume_percent: i32) {
    let volume = volume_percent.clamp(0, 400);
    if volume == 100 {
        return;
    }
    for sample in samples {
        let scaled = (i32::from(*sample) * volume) / 100;
        *sample = scaled.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
    }
}

fn volume_for_path(path: &Path, volumes: &IniDocument) -> i32 {
    let sample_name = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    let folder_name = path
        .parent()
        .and_then(Path::file_name)
        .and_then(|value| value.to_str())
        .unwrap_or_default();

    for key in [sample_name, folder_name, "Padrao"] {
        if key.is_empty() {
            continue;
        }
        if let Some(value) = volumes.get("Volume", key) {
            return value.parse::<i32>().unwrap_or(100).clamp(0, 400);
        }
    }
    100
}

#[derive(Debug, Default, Clone)]
struct IniDocument {
    entries: HashMap<(String, String), String>,
}

impl IniDocument {
    fn read(path: &Path) -> Self {
        fs::read_to_string(path)
            .map(|contents| Self::parse(&contents))
            .unwrap_or_default()
    }

    fn parse(contents: &str) -> Self {
        let mut document = Self::default();
        let mut section = String::new();

        for raw_line in contents.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with(';') {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                section = line[1..line.len() - 1].trim().to_ascii_lowercase();
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                document.entries.insert(
                    (section.clone(), key.trim().to_ascii_lowercase()),
                    value.trim().to_owned(),
                );
            }
        }
        document
    }

    fn get(&self, section: &str, key: &str) -> Option<&str> {
        self.entries
            .get(&(section.to_ascii_lowercase(), key.to_ascii_lowercase()))
            .map(String::as_str)
    }

    fn merge(&mut self, override_values: Self) {
        self.entries.extend(override_values.entries);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pascal_numbered_files_replace_only_the_first_hash_run() {
        assert_eq!(
            numbered_file("Sons/Som-##.wav", 7),
            "Sons/Som-07.wav"
        );
        assert_eq!(
            numbered_file("Falas/Fala-##-a.wav", 16),
            "Falas/Fala-16-a.wav"
        );
        assert_eq!(numbered_file("Embutidos/Musica.wav", 2), "Embutidos/Musica.wav");
    }

    #[test]
    fn fighter_count_follows_the_pascal_lutadores_data_pattern() {
        let root = std::env::temp_dir().join(format!(
            "street-chaves-audio-count-{}",
            std::process::id()
        ));
        let data = root.join("Dados/Lutadores");
        fs::create_dir_all(&data).unwrap();
        fs::write(data.join("C01.ini"), "[Geral]").unwrap();
        fs::write(data.join("C02.ini"), "[Geral]").unwrap();
        fs::write(data.join("C04.ini"), "[Geral]").unwrap();

        assert_eq!(
            count_numbered_files(&root, "Dados/Lutadores/C##.ini"),
            2,
            "Pascal stops at the first missing fighter INI"
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn user_volume_settings_override_data_defaults_case_insensitively() {
        let mut values = IniDocument::parse("[Volume]\nPadrao=100\nSons=80\nSom-05=65\n");
        values.merge(IniDocument::parse("[volume]\nsom-05=42\n"));
        assert_eq!(
            volume_for_path(Path::new("Sons/Som-05.wav"), &values),
            42
        );
        assert_eq!(
            volume_for_path(Path::new("Sons/Som-06.wav"), &values),
            80
        );
        assert_eq!(
            volume_for_path(Path::new("Falas/Fala-01-a.wav"), &values),
            100
        );
    }

    #[test]
    fn volume_scaling_clamps_and_matches_integer_percent_math() {
        let mut samples = [30_000_i16, -30_000_i16, 100_i16, -100_i16];
        apply_volume(&mut samples, 200);
        assert_eq!(samples, [i16::MAX, i16::MIN, 200, -200]);

        apply_volume(&mut samples, -10);
        assert_eq!(samples, [0, 0, 0, 0]);
    }
}
