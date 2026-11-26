//! Pascal resource-bank adapter over RustMultiConsoleFoundation audio.
//! StreetChaves owns INI parsing and sound IDs; Foundation owns WAV decode, mixer and device.

use crate::fight_simulation::FightSimulation;
use foundation_runtime::native_sdl::Sdl3Host;
use foundation_runtime::{AudioClipId, AudioRequest, RuntimeError, RuntimeHost};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const EFFECTS: &str = "Sons/Som-##.wav";
const MUSIC: &str = "Musicas/Musica-##.wav";
const MENU: &str = "Embutidos/Musica.wav";
const VICTORY: &str = "Falas/Fala-##-a.wav,Falas/Fala-##-b.wav";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioBankId(usize);

#[derive(Debug, Clone)]
struct AudioBank {
    clips: Vec<Option<AudioClipId>>,
    looped: bool,
}

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

/// Preserves Pascal resource numbering while delegating all playback to Foundation.
#[derive(Debug, Default)]
pub struct FoundationAudioOutput {
    banks: Vec<AudioBank>,
}

impl FoundationAudioOutput {
    pub fn new() -> Self {
        Self::default()
    }

    fn create_bank(&mut self, looped: bool) -> AudioBankId {
        let id = AudioBankId(self.banks.len());
        self.banks.push(AudioBank {
            clips: Vec::new(),
            looped,
        });
        id
    }

    fn add_empty_slot(&mut self, bank: AudioBankId) -> Option<usize> {
        let record = self.banks.get_mut(bank.0)?;
        let index = record.clips.len();
        record.clips.push(None);
        Some(index)
    }

    fn add_wav(
        &mut self,
        host: &mut Sdl3Host,
        bank: AudioBankId,
        path: &Path,
        volume: i32,
        warnings: &mut Vec<String>,
    ) -> Option<usize> {
        let index = self.add_empty_slot(bank)?;
        if path.is_file() {
            match host.load_wav(path, volume) {
                Ok(clip) => self.banks[bank.0].clips[index] = Some(clip),
                Err(error) => warnings.push(format!("{}: {error}", path.display())),
            }
        }
        Some(index)
    }

    fn replace_wav(
        &mut self,
        host: &mut Sdl3Host,
        bank: AudioBankId,
        index: usize,
        path: &Path,
        volume: i32,
    ) -> bool {
        let Some(existing) = self
            .banks
            .get(bank.0)
            .and_then(|b| b.clips.get(index))
            .copied()
        else {
            return false;
        };
        if let Some(old) = existing {
            let _ = host.audio(&AudioRequest::StopClip { clip: old });
        }
        self.banks[bank.0].clips[index] = None;
        if path.is_file() {
            match host.load_wav(path, volume) {
                Ok(clip) => self.banks[bank.0].clips[index] = Some(clip),
                Err(_) => return false,
            }
        }
        true
    }

    pub fn play<H: RuntimeHost>(&self, host: &mut H, bank: AudioBankId, index: usize) -> bool {
        let Some(record) = self.banks.get(bank.0) else {
            return false;
        };
        let Some(Some(clip)) = record.clips.get(index) else {
            return false;
        };
        host.audio(&AudioRequest::PlayClip {
            clip: *clip,
            looped: record.looped,
        })
        .is_ok()
    }

    pub fn stop<H: RuntimeHost>(&self, host: &mut H, bank: AudioBankId, index: usize) -> bool {
        let Some(Some(clip)) = self.banks.get(bank.0).and_then(|b| b.clips.get(index)) else {
            return false;
        };
        host.audio(&AudioRequest::StopClip { clip: *clip }).is_ok()
    }

    pub fn bank_len(&self, bank: AudioBankId) -> Option<usize> {
        self.banks.get(bank.0).map(|record| record.clips.len())
    }

    /// Read local resource INIs, preserving missing-file slots and Pascal's bank order.
    pub fn load_pascal_resources(
        &mut self,
        host: &mut Sdl3Host,
        game_root: impl AsRef<Path>,
    ) -> PascalAudioLayout {
        let root = game_root.as_ref().to_path_buf();
        let resources = IniDocument::read(&root.join("Dados/Recursos.ini"));
        let mut volumes = IniDocument::read(&root.join("Dados/Som.ini"));
        volumes.merge(IniDocument::read(&root.join("Chaves.ini")));

        let effects_pattern = resources
            .get("Sons", "Arquivo")
            .unwrap_or(EFFECTS)
            .to_owned();
        let music_pattern = resources
            .get("Musicas", "Arquivo")
            .unwrap_or(MUSIC)
            .to_owned();
        let menu_music = resources.get("Musicas", "Menu").unwrap_or(MENU).to_owned();
        let victory_variants = resources
            .get("Falas", "Variantes")
            .unwrap_or(VICTORY)
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let mut warnings = Vec::new();

        let effects = self.create_bank(false);
        for number in 1..=255 {
            let path = rooted_path(&root, &numbered_file(&effects_pattern, number));
            if !path.is_file() {
                break;
            }
            let volume = volume_for_path(&path, &volumes);
            let _ = self.add_wav(host, effects, &path, volume, &mut warnings);
        }
        // Pascal leaves one final empty slot after the first missing numbered effect file.
        if self.bank_len(effects).unwrap_or(0) > 0 {
            let _ = self.add_empty_slot(effects);
        }

        let music = self.create_bank(true);
        let menu_path = rooted_path(&root, &menu_music);
        let _ = self.add_wav(
            host,
            music,
            &menu_path,
            volume_for_path(&menu_path, &volumes),
            &mut warnings,
        );

        let mut fight_music_count = 0;
        for number in 1..=255 {
            let path = rooted_path(&root, &numbered_file(&music_pattern, number));
            if !path.is_file() {
                break;
            }
            fight_music_count += 1;
        }

        let victory = self.create_bank(false);
        let fighter_pattern = resources
            .get("Lutadores", "Dados")
            .unwrap_or("Dados/Lutadores/C##.ini");
        let configured_fighters = resources
            .get("Falas", "Quantidade")
            .and_then(|v| v.parse::<usize>().ok())
            .or_else(|| {
                resources
                    .get("Lutadores", "Quantidade")
                    .and_then(|v| v.parse::<usize>().ok())
            });
        let fighter_count =
            configured_fighters.unwrap_or_else(|| count_numbered_files(&root, fighter_pattern));

        // Keep fighter-first / variant-second offsets even when a voice file is absent.
        for fighter in 1..=fighter_count {
            for pattern in &victory_variants {
                let path = rooted_path(&root, &numbered_file(pattern, fighter));
                let volume = volume_for_path(&path, &volumes);
                let _ = self.add_wav(host, victory, &path, volume, &mut warnings);
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

    pub fn load_fight_music(
        &mut self,
        host: &mut Sdl3Host,
        layout: &PascalAudioLayout,
        number: usize,
    ) -> bool {
        if number == 0 || number > layout.fight_music_count {
            return false;
        }
        let path = rooted_path(
            &layout.root,
            &numbered_file(&layout.fight_music_pattern, number),
        );
        self.replace_wav(
            host,
            layout.music,
            0,
            &path,
            volume_for_path(&path, &layout.volumes),
        )
    }

    /// Consume the pending sound immediately after a logical tick, never from rendering.
    pub fn play_pending_fight_sound(
        &self,
        host: &mut impl RuntimeHost,
        layout: &PascalAudioLayout,
        fight: &mut FightSimulation,
    ) -> bool {
        let Some(sound_id) = fight.state.pending_sound.filter(|id| *id != 0) else {
            return false;
        };
        let slot = usize::from(sound_id);
        if !self
            .bank_len(layout.effects)
            .is_some_and(|count| slot < count)
        {
            return false;
        }
        let _ = fight.take_pending_sound();
        self.play(host, layout.effects, slot)
    }

    pub fn play_music(&self, host: &mut Sdl3Host, layout: &PascalAudioLayout) -> bool {
        self.play(host, layout.music, 0)
    }

    pub fn stop_music(&self, host: &mut Sdl3Host, layout: &PascalAudioLayout) -> bool {
        self.stop(host, layout.music, 0)
    }

    pub fn play_victory_voice(
        &self,
        host: &mut Sdl3Host,
        layout: &PascalAudioLayout,
        index: usize,
    ) -> bool {
        self.play(host, layout.victory, index)
    }

    pub fn dispatch(
        &self,
        host: &mut impl RuntimeHost,
        request: &AudioRequest,
    ) -> Result<(), RuntimeError> {
        host.audio(request)
    }
}

fn count_numbered_files(root: &Path, pattern: &str) -> usize {
    (1..=255)
        .take_while(|n| rooted_path(root, &numbered_file(pattern, *n)).is_file())
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

fn numbered_file(pattern: &str, number: usize) -> String {
    let Some(start) = pattern.find('#') else {
        return pattern.to_owned();
    };
    let width = pattern[start..].chars().take_while(|c| *c == '#').count();
    let end = start + width;
    format!("{}{:0width$}{}", &pattern[..start], number, &pattern[end..])
}

fn volume_for_path(path: &Path, volumes: &IniDocument) -> i32 {
    let name = path
        .file_stem()
        .and_then(|v| v.to_str())
        .unwrap_or_default();
    let folder = path
        .parent()
        .and_then(Path::file_name)
        .and_then(|v| v.to_str())
        .unwrap_or_default();
    for key in [name, folder, "Padrao"] {
        if !key.is_empty() {
            if let Some(value) = volumes.get("Volume", key) {
                return value.parse::<i32>().unwrap_or(100).clamp(0, 400);
            }
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
            .map(|s| Self::parse(&s))
            .unwrap_or_default()
    }

    fn parse(contents: &str) -> Self {
        let mut result = Self::default();
        let mut section = String::new();
        for raw in contents.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with(';') {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                section = line[1..line.len() - 1].trim().to_ascii_lowercase();
            } else if let Some((key, value)) = line.split_once('=') {
                result.entries.insert(
                    (section.clone(), key.trim().to_ascii_lowercase()),
                    value.trim().to_owned(),
                );
            }
        }
        result
    }

    fn get(&self, section: &str, key: &str) -> Option<&str> {
        self.entries
            .get(&(section.to_ascii_lowercase(), key.to_ascii_lowercase()))
            .map(String::as_str)
    }

    fn merge(&mut self, overrides: Self) {
        self.entries.extend(overrides.entries);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effect_indices_preserve_pascal_zero_based_bank_numbering() {
        assert_eq!(numbered_file(EFFECTS, 1), "Sons/Som-01.wav");
        assert_eq!(numbered_file(EFFECTS, 11), "Sons/Som-11.wav");
    }

    #[test]
    fn ini_volume_override_is_case_insensitive_and_takes_precedence() {
        let mut values = IniDocument::parse("[Volume]\nPadrao=100\nSons=80\nSom-05=65\n");
        values.merge(IniDocument::parse("[volume]\nsom-05=42\n"));
        assert_eq!(volume_for_path(Path::new("Sons/Som-05.wav"), &values), 42);
        assert_eq!(volume_for_path(Path::new("Sons/Som-06.wav"), &values), 80);
        assert_eq!(
            volume_for_path(Path::new("Falas/Fala-01-a.wav"), &values),
            100
        );
    }
}
