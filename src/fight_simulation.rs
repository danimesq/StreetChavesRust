//! First concrete, renderer-free fight orchestrator.
use crate::chaves_specials::{ChavesSpecialCommand, ChavesSpecialContext, ChavesSpecialOutcome};
use crate::custom_character::{CustomCharacterPackage, CustomCharacterRegistry};
use crate::custom_character_runtime::CustomCharacterDefinition;
use std::collections::BTreeMap;
use crate::combat_resolution::{
    resolve_melee_contact, resolve_melee_contact_after_guard_break, CombatResolution,
    CombatResolutionContext,
};
use crate::combat_runtime::{
    C01FrameResult, CombatState, VictoryAnimationContext, ACTION_AIR_STRONG_KICK,
    ACTION_AIR_STRONG_PUNCH, ACTION_AIR_WEAK_KICK, ACTION_AIR_WEAK_PUNCH, ACTION_CROUCHING,
    ACTION_CROUCH_STRONG_KICK, ACTION_CROUCH_STRONG_PUNCH, ACTION_CROUCH_WEAK_KICK,
    ACTION_CROUCH_WEAK_PUNCH, ACTION_DEFENDING, ACTION_IDLE, ACTION_JUMPING, ACTION_LOW_DEFENDING,
    ACTION_POSE, ACTION_SPECIAL1, ACTION_SPECIAL2, ACTION_SPECIAL3, ACTION_SPECIAL4,
    ACTION_SPECIAL_FINAL, ACTION_STRONG_KICK, ACTION_STRONG_PUNCH, ACTION_VICTORY, ACTION_WALKING,
    ACTION_WEAK_KICK, ACTION_WEAK_PUNCH, SIDE_LEFT,
};
use crate::command_recognition::{AttackButton, CommandRecognizer};
use crate::effect_runtime::{EffectTickContext, EffectTickResult, EFFECT_SIDE_RIGHT};
use crate::fighter_box_data::{load_fighter_frame_boxes, FighterFrameBoxes};
use crate::fighter_command_data::{embedded_character_command_rules, CharacterCommandRules};
use crate::input::{Direction, InputFrame};
use crate::pascal_replay::PascalRandomSource;
use crate::rot_runtime::{RotDocument, RotExecutionFrame, RotExecutionHost, RotValue};
use crate::runtime::SimulationTick;
use crate::Fpc322Rng;
use crate::{ActionId, PascalAttackState};

/// Bridge from Pascal ROT field/helper names to the authoritative fight state.
struct RotCombatHost<'a> {
    state: &'a mut CombatState,
    rng: &'a mut Fpc322Rng,
    player_index: usize,
    charge_gain: f64,
    charge_max: f64,
    ground_y: f64,
    common: &'a RotDocument,
    recursion_depth: usize,
}

fn rot_int(value: i64) -> RotValue {
    RotValue::Integer { value, wide: false }
}

fn rot_value_int(value: &RotValue) -> Result<i64, String> {
    match value {
        RotValue::Integer { value, .. } => Ok(*value),
        _ => Err("expected an integer argument".to_owned()),
    }
}

fn rot_value_real(value: &RotValue) -> Result<f64, String> {
    match value {
        RotValue::Real(value) => Ok(*value),
        RotValue::Integer { value, .. } => Ok(*value as f64),
        _ => Err("expected a numeric argument".to_owned()),
    }
}

fn rot_value_bool(value: &RotValue) -> Result<bool, String> {
    match value {
        RotValue::Boolean(value) => Ok(*value),
        _ => Err("expected a Boolean value".to_owned()),
    }
}

impl RotCombatHost<'_> {
    fn fighter_index(&self, base: char) -> Result<usize, String> {
        match base.to_ascii_uppercase() {
            'P' => Ok(self.player_index),
            'O' => Ok(1 - self.player_index),
            _ => Err(format!("unknown fighter base {base}")),
        }
    }

    fn fighter(&self, base: char) -> Result<&crate::combat_runtime::FighterState, String> {
        let index = self.fighter_index(base)?;
        Ok(&self.state.fighters[index])
    }
}

impl RotCombatHost<'_> {
    /// Recursively executes the same common/character/common-end action lifecycle
    /// as Pascal AcaoJogador, sharing the live fighters, effects and RNG cursor.
    fn execute_nested_action(
        &mut self,
        fighter_index: usize,
        command_code: u8,
    ) -> Result<(), String> {
        const MAX_NESTED_ACTION_DEPTH: usize = 16;
        if self.recursion_depth >= MAX_NESTED_ACTION_DEPTH {
            return Err(format!(
                "AcaoJogador recursion limit ({MAX_NESTED_ACTION_DEPTH}) exceeded"
            ));
        }
        let character_id = self.state.fighters[fighter_index].character_id;
        if character_id >= 16 {
            let jog = (fighter_index + 1) as u8;
            let previous_player_index = self.player_index;
            self.player_index = fighter_index;
            self.recursion_depth += 1;
            let result = execute_rot_section(self.common, "Acao", jog, command_code, self)
                .and_then(|()| execute_rot_section(self.common, "AcaoFim", jog, command_code, self));
            self.recursion_depth -= 1;
            self.player_index = previous_player_index;
            return result;
        }
        let (filename, source) = match character_id {
            1 => ("C01.rot", include_str!("../research/pascal-rot/C01.rot")),
            2 => ("C02.rot", include_str!("../research/pascal-rot/C02.rot")),
            3 => ("C03.rot", include_str!("../research/pascal-rot/C03.rot")),
            4 => ("C04.rot", include_str!("../research/pascal-rot/C04.rot")),
            5 => ("C05.rot", include_str!("../research/pascal-rot/C05.rot")),
            6 => ("C06.rot", include_str!("../research/pascal-rot/C06.rot")),
            7 => ("C07.rot", include_str!("../research/pascal-rot/C07.rot")),
            8 => ("C08.rot", include_str!("../research/pascal-rot/C08.rot")),
            9 => ("C09.rot", include_str!("../research/pascal-rot/C09.rot")),
            10 => ("C10.rot", include_str!("../research/pascal-rot/C10.rot")),
            11 => ("C11.rot", include_str!("../research/pascal-rot/C11.rot")),
            12 => ("C12.rot", include_str!("../research/pascal-rot/C12.rot")),
            13 => ("C13.rot", include_str!("../research/pascal-rot/C13.rot")),
            14 => ("C14.rot", include_str!("../research/pascal-rot/C14.rot")),
            15 => ("C15.rot", include_str!("../research/pascal-rot/C15.rot")),
            _ => {
                return Err(format!(
                    "AcaoJogador has no ROT route for C{character_id:02}"
                ))
            }
        };
        let character = RotDocument::from_str(filename, source).map_err(|e| e.to_string())?;
        let jog = (fighter_index + 1) as u8;
        // Nested AcaoJogador changes the host's P/O perspective to its target
        // while preserving the same underlying fighter array and RNG cursor.
        let previous_player_index = self.player_index;
        self.player_index = fighter_index;
        self.recursion_depth += 1;
        let result = execute_rot_section(self.common, "Acao", jog, command_code, self)
            .and_then(|()| execute_rot_section(&character, "Acao", jog, command_code, self))
            .and_then(|()| execute_rot_section(self.common, "AcaoFim", jog, command_code, self));
        self.recursion_depth -= 1;
        self.player_index = previous_player_index;
        result
    }
}

impl RotExecutionHost for RotCombatHost<'_> {
    fn read_fighter_field(&self, base: char, name: &str) -> Result<RotValue, String> {
        let fighter = self.fighter(base)?;
        let value = match name.to_ascii_lowercase().as_str() {
            "personagem" => rot_int(i64::from(fighter.character_id)),
            "acao" => rot_int(i64::from(fighter.action_id)),
            "quadro" => rot_int(i64::from(fighter.sprite_frame)),
            "quadroacao" => rot_int(i64::from(fighter.action_frame)),
            "duracaoacao" => rot_int(i64::from(fighter.action_duration)),
            "lado" => rot_int(i64::from(fighter.facing)),
            "forcagolpe" => rot_int(i64::from(fighter.force_strength)),
            "defendendo" => RotValue::Boolean(fighter.defending),
            "invulneravel" => RotValue::Boolean(fighter.invulnerable),
            "ehchute" => RotValue::Boolean(fighter.is_kick),
            "noar" => RotValue::Boolean(fighter.in_air),
            "recuo" => rot_int(i64::from(fighter.recoil)),
            "vely" => rot_int(i64::from(fighter.vertical_velocity)),
            "x" => RotValue::Real(fighter.x),
            "y" => RotValue::Real(fighter.y),
            "velocidade" => RotValue::Real(fighter.speed),
            "distanciaia" => rot_int(i64::from(fighter.distance_ai)),
            "vida" => RotValue::Real(fighter.health),
            "poder" => rot_int(i64::from(fighter.power)),
            "cargapoder" => RotValue::Real(fighter.charge_power),
            "dirpulo" => rot_int(i64::from(fighter.jump_direction)),
            "tempobaixo" => rot_int(i64::from(fighter.low_time)),
            "tempotras" => rot_int(i64::from(fighter.back_time)),
            "seqestado" => rot_int(i64::from(fighter.sequence_state)),
            "seqtempo" => rot_int(i64::from(fighter.sequence_timer)),
            "cargasoco" => rot_int(i64::from(fighter.punch_charge)),
            "cargachute" => rot_int(i64::from(fighter.kick_charge)),
            "acertos" => rot_int(i64::from(fighter.hit_count)),
            "tempoacertos" => rot_int(i64::from(fighter.hit_timer)),
            "danogolpe" => rot_int(i64::from(fighter.attack_damage)),
            "velangular" => rot_int(i64::from(fighter.angular_velocity)),
            "golpeativo" => RotValue::Boolean(fighter.attack_active),
            "andandofrente" => RotValue::Boolean(fighter.walking_forward),
            _ => return Err(format!("{base}.{name} is not mapped to live FighterState")),
        };
        Ok(value)
    }

    fn write_fighter_field(
        &mut self,
        base: char,
        name: &str,
        value: RotValue,
    ) -> Result<(), String> {
        let index = self.fighter_index(base)?;
        let fighter = &mut self.state.fighters[index];
        match name.to_ascii_lowercase().as_str() {
            "personagem" => fighter.character_id = rot_value_int(&value)? as u8,
            "acao" => fighter.action_id = u16::from(rot_value_int(&value)? as u8),
            "quadro" => fighter.sprite_frame = i32::from(rot_value_int(&value)? as u8),
            "quadroacao" => fighter.action_frame = (rot_value_int(&value)? as i16) as u16,
            "duracaoacao" => fighter.action_duration = (rot_value_int(&value)? as i16) as u16,
            "lado" => fighter.facing = rot_value_int(&value)? as u8,
            "forcagolpe" => fighter.force_strength = rot_value_int(&value)? as u8,
            "defendendo" => fighter.defending = rot_value_bool(&value)?,
            "invulneravel" => fighter.invulnerable = rot_value_bool(&value)?,
            "ehchute" => fighter.is_kick = rot_value_bool(&value)?,
            "noar" => fighter.in_air = rot_value_bool(&value)?,
            "recuo" => fighter.recoil = rot_value_int(&value)? as u8,
            "vely" => fighter.vertical_velocity = rot_value_int(&value)? as i8,
            "x" => fighter.x = rot_value_real(&value)?,
            "y" => fighter.y = rot_value_real(&value)?,
            "velocidade" => fighter.speed = rot_value_real(&value)?,
            "distanciaia" => fighter.distance_ai = rot_value_int(&value)? as i32,
            "vida" => fighter.health = rot_value_real(&value)?,
            "poder" => fighter.power = rot_value_int(&value)? as u16,
            "cargapoder" => fighter.charge_power = rot_value_real(&value)?,
            "dirpulo" => fighter.jump_direction = rot_value_int(&value)? as u8,
            "tempobaixo" => fighter.low_time = rot_value_int(&value)? as u8,
            "tempotras" => fighter.back_time = rot_value_int(&value)? as u8,
            "seqestado" => fighter.sequence_state = rot_value_int(&value)? as u8,
            "seqtempo" => fighter.sequence_timer = rot_value_int(&value)? as u8,
            "cargasoco" => fighter.punch_charge = rot_value_int(&value)? as u8,
            "cargachute" => fighter.kick_charge = rot_value_int(&value)? as u8,
            "acertos" => fighter.hit_count = rot_value_int(&value)? as u8,
            "tempoacertos" => fighter.hit_timer = rot_value_int(&value)? as u8,
            "danogolpe" => fighter.attack_damage = rot_value_int(&value)? as u8,
            "velangular" => fighter.angular_velocity = rot_value_int(&value)? as i16,
            "golpeativo" => fighter.attack_active = rot_value_bool(&value)?,
            "andandofrente" => fighter.walking_forward = rot_value_bool(&value)?,
            _ => {
                return Err(format!(
                    "{base}.{name} is not writable in live FighterState"
                ))
            }
        }
        Ok(())
    }

    fn read_identifier(&self, name: &str) -> Option<RotValue> {
        // Numeric constants copied from ChavesTipos.pas at the pinned source revision.
        const CONSTANTS: &[(&str, i64)] = &[
            ("acParado", 1),
            ("acAndando", 2),
            ("acSocoFraco", 3),
            ("acChuteFraco", 4),
            ("acSocoForte", 5),
            ("acChuteForte", 6),
            ("acAgachado", 7),
            ("acAgSocoFraco", 8),
            ("acAgChuteFraco", 9),
            ("acAgSocoForte", 10),
            ("acAgChuteForte", 11),
            ("acPulo", 12),
            ("acArSocoFraco", 13),
            ("acArChuteFraco", 14),
            ("acArSocoForte", 15),
            ("acArChuteForte", 16),
            ("acApanhaAlto", 17),
            ("acApanhaBaixo", 18),
            ("acDerrubado", 19),
            ("acQuicando", 20),
            ("acDefesa", 21),
            ("acDefesaBaixa", 22),
            ("acVitoria", 23),
            ("acPose", 24),
            ("acEspecial1", 25),
            ("acEspecial2", 26),
            ("acEspecial3", 27),
            ("acEspecial4", 28),
            ("acEspecial5", 29),
            ("acEspecial6", 30),
            ("acEspecial7", 31),
            ("cmEsquerda", 1),
            ("cmDireita", 2),
            ("cmSocoFraco", 3),
            ("cmChuteFraco", 4),
            ("cmSocoForte", 5),
            ("cmChuteForte", 6),
            ("cmAgachar", 7),
            ("cmApanharAlto", 8),
            ("cmApanharBaixo", 9),
            ("cmDerrubar", 10),
            ("cmBaterNoChao", 11),
            ("cmDefender", 12),
            ("cmDefenderBaixo", 13),
            ("cmVitoria", 14),
            ("cmPose", 15),
            ("cmEsp1Fraco", 16),
            ("cmEsp1Forte", 17),
            ("cmEsp2Fraco", 18),
            ("cmEsp2Forte", 19),
            ("cmEsp3Fraco", 20),
            ("cmEsp3Forte", 21),
            ("cmEsp4Fraco", 22),
            ("cmEsp4Forte", 23),
            ("cmEspFinal", 24),
            ("cmEspExtra1", 25),
            ("cmEspExtra2", 26),
            ("pnChaves", 1),
            ("pnSeuMadruga", 2),
            ("pnChiquinha", 3),
            ("pnQuico", 4),
            ("pnDonaFlorinda", 5),
            ("pnProfGirafales", 6),
            ("pnSrBarriga", 7),
            ("pnBruxa", 8),
            ("pnPaty", 9),
            ("pnNhonho", 10),
            ("pnGodines", 11),
            ("pnSrFurtado", 12),
            ("pnJaiminho", 13),
            ("pnDonaNeves", 14),
            ("pnGloria", 15),
            ("ldDireita", 1),
            ("ldEsquerda", 2),
            ("fgNenhum", 0),
            ("fgFraco", 1),
            ("fgForte", 2),
            ("fgEspecial", 3),
            ("puloNenhum", 0),
            ("puloReto", 1),
            ("puloEsquerda", 2),
            ("puloDireita", 3),
            ("sndTempoEsgotado", 4),
            ("sndGolpeFraco", 5),
            ("sndGolpeForte", 6),
            ("sndAcertoFraco", 7),
            ("sndAcertoForte", 8),
            ("sndAcertoEspecial", 9),
            ("sndDefesa", 10),
            ("sndItem", 11),
            ("sndQueda", 12),
            ("sndPoder", 13),
            ("sndEspFinal", 19),
        ];
        if let Some((_, value)) = CONSTANTS
            .iter()
            .find(|(symbol, _)| symbol.eq_ignore_ascii_case(name))
        {
            return Some(rot_int(*value));
        }
        match name.to_ascii_lowercase().as_str() {
            "cargamax" => Some(RotValue::Real(self.charge_max)),
            "ganhocarga" => Some(RotValue::Real(self.charge_gain)),
            "chaoy" => Some(RotValue::Real(self.ground_y)),
            "camerax" => Some(RotValue::Real(self.state.camera_x)),
            "sompendente" => Some(rot_int(i64::from(self.state.pending_sound.unwrap_or(0)))),
            "flashtela" => Some(rot_int(i64::from(self.state.screen_flash))),
            "tremortela" => Some(rot_int(i64::from(self.state.screen_shake))),
            _ => None,
        }
    }

    fn write_identifier(&mut self, name: &str, value: RotValue) -> Result<(), String> {
        match name.to_ascii_lowercase().as_str() {
            "sompendente" => self.state.pending_sound = Some(rot_value_int(&value)? as u16),
            "flashtela" => self.state.screen_flash = rot_value_int(&value)? as u8,
            "tremortela" => self.state.screen_shake = rot_value_int(&value)? as u16,
            _ => {
                return Err(format!(
                    "global identifier {name} is not mapped to live game state"
                ))
            }
        }
        Ok(())
    }

    fn call_function(&mut self, name: &str, arguments: &[RotValue]) -> Result<RotValue, String> {
        match name.to_ascii_lowercase().as_str() {
            "random" => {
                if arguments.len() != 1 {
                    return Err("Random expects exactly one argument".to_owned());
                }
                let range = i32::try_from(rot_value_int(&arguments[0])?)
                    .map_err(|_| "Random range is outside LongInt".to_owned())?;
                if range <= 0 {
                    return Err(format!("Random range must be positive, got {range}"));
                }
                let value = match self.rng.draw_longint(range) {
                    Ok(value) => value,
                    Err(never) => match never {},
                };
                Ok(rot_int(i64::from(value)))
            }
            "existeefeito" => {
                if arguments.len() != 2 {
                    return Err("ExisteEfeito expects two arguments".to_owned());
                }
                let effect_type = rot_value_int(&arguments[0])? as u8;
                let owner = rot_value_int(&arguments[1])? as u8;
                Ok(RotValue::Boolean(
                    self.state
                        .effects
                        .contains_type_for_owner(effect_type, owner),
                ))
            }
            "criaefeito" => {
                if arguments.len() != 5 {
                    return Err("CriaEfeito expects five arguments".to_owned());
                }
                let effect_type = rot_value_int(&arguments[0])? as u8;
                let variant = match &arguments[1] {
                    RotValue::String(value) if value.is_empty() => None,
                    RotValue::String(value) if value.len() == 1 => value.chars().next(),
                    _ => return Err("CriaEfeito variant must be empty, A or B".to_owned()),
                };
                let owner = rot_value_int(&arguments[2])? as u8;
                if owner > 2 {
                    return Err(format!("CriaEfeito owner {owner} is outside 0..=2"));
                }
                let owner_facing = if owner == 0 {
                    EFFECT_SIDE_RIGHT
                } else {
                    self.state.fighters[usize::from(owner - 1)].facing
                };
                let x = rot_value_real(&arguments[3])?;
                let y = rot_value_real(&arguments[4])?;
                let state = &mut *self.state;
                let rng = &mut *self.rng;
                state
                    .effects
                    .spawn(effect_type, variant, owner, owner_facing, x, y, rng)
                    .map_err(|error| error.to_string())?;
                Ok(rot_int(0))
            }
            "acaojogador" => {
                if arguments.len() != 2 {
                    return Err("AcaoJogador expects two arguments".to_owned());
                }
                let player = rot_value_int(&arguments[0])?;
                let command = rot_value_int(&arguments[1])?;
                if !(1..=2).contains(&player) {
                    return Err(format!("AcaoJogador Jog must be 1 or 2, got {player}"));
                }
                if !(1..=26).contains(&command) {
                    return Err(format!(
                        "AcaoJogador Codigo must be in 1..=26, got {command}"
                    ));
                }
                self.execute_nested_action(player as usize - 1, command as u8)?;
                Ok(rot_int(0))
            }
            "empurra" => {
                if arguments.len() != 2 {
                    return Err("Empurra expects two arguments".to_owned());
                }
                let jog = rot_value_int(&arguments[0])?;
                if !(1..=2).contains(&jog) {
                    return Err(format!("Empurra Jog must be 1 or 2, got {jog}"));
                }
                let dx = rot_value_real(&arguments[1])?;
                self.state.push_fighter((jog - 1) as usize, dx);
                Ok(rot_int(0))
            }
            _ => Err(format!(
                "ROT helper {name} is not implemented by the fight host"
            )),
        }
    }
}

fn execute_rot_section(
    document: &RotDocument,
    section: &str,
    jog: u8,
    command_code: u8,
    host: &mut RotCombatHost<'_>,
) -> Result<(), String> {
    let program = document
        .parse_optional(section)
        .map_err(|error| error.to_string())?;
    let Some(program) = program else {
        return Ok(());
    };
    let mut frame = RotExecutionFrame::new(&program, jog, i32::from(command_code))
        .map_err(|error| error.to_string())?;
    frame
        .execute_with_host(&program, host)
        .map_err(|error| error.to_string())
}

/// Configuration for the already-audited C01 entry and frame executors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FightSimulationConfig {
    pub special: ChavesSpecialContext,
    pub frame: VictoryAnimationContext,
    /// Camera Y used by the existing effect culling contract.
    pub effect_camera_y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandMatch {
    pub fighter_index: usize,
    pub button: AttackButton,
    pub command_code: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnhandledCommandSource {
    CharacterButtonRule,
    BasicButtonAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnhandledCommand {
    pub fighter_index: usize,
    pub button: AttackButton,
    pub command_code: u8,
    pub source: UnhandledCommandSource,
}

/// One headless simulation tick's observable work.
#[derive(Debug, Clone, PartialEq)]
pub enum CustomCharacterEvent {
    SpawnEffect {
        fighter_index: usize,
        effect_id: u8,
        variant: Option<char>,
        x: f64,
        y: f64,
    },
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FightTickReport {
    pub tick_index: u64,
    /// Backwards-compatible shorthand for the player-one input sample.
    pub input: InputFrame,
    pub fighter_inputs: [InputFrame; 2],
    pub command_matches: Vec<CommandMatch>,
    /// Commands requiring common/non-C01 action implementations not yet ported.
    pub unhandled_commands: Vec<UnhandledCommand>,
    pub sequence_transitions: Vec<(usize, u8, u8)>,
    pub special_outcomes: Vec<(usize, ChavesSpecialOutcome)>,
    /// Character action sections executed by the pinned ROT interpreter.
    pub rot_action_executions: Vec<(usize, u8, u8)>,
    /// Character-specific [Quadro] scripts executed by the live ROT interpreter.
    pub rot_frame_executions: Vec<(usize, u8)>,
    /// Fighters whose current boxes were successfully updated after both [Quadro] sections.
    pub box_updates: Vec<usize>,
    pub c01_frames: Vec<(usize, C01FrameResult)>,
    pub effects: EffectTickResult,
    /// Declarative effects requested by user-installed character timelines.
    pub custom_character_events: Vec<CustomCharacterEvent>,
    /// Custom fighter IDs whose [Quadro]-equivalent timeline ran this tick.
    pub custom_character_frame_executions: Vec<(usize, u8)>,
    pub errors: Vec<String>,
}

/// Concrete SimulationTick implementation over the existing combat/effect state.
#[derive(Debug, Clone)]
pub struct FightSimulation {
    pub state: CombatState,
    config: FightSimulationConfig,
    rng: Fpc322Rng,
    pending_specials: Vec<(usize, ChavesSpecialCommand)>,
    command_rules: [Option<CharacterCommandRules>; 2],
    command_recognizers: [CommandRecognizer; 2],
    command_data_errors: Vec<String>,
    /// INI frame/box rows, loaded once per fighter and kept in source order.
    frame_boxes: [Option<FighterFrameBoxes>; 2],
    /// Installed user packages and their validated, bounded runtime definitions.
    custom_character_registry: CustomCharacterRegistry,
    custom_character_definitions: BTreeMap<u8, CustomCharacterDefinition>,
    last_report: FightTickReport,
}

impl FightSimulation {
    /// Build a deterministic headless fight and load real per-character command tables.
    pub fn new(state: CombatState, config: FightSimulationConfig, rng_seed: i32) -> Self {
        let mut command_rules: [Option<CharacterCommandRules>; 2] = [None, None];
        let mut frame_boxes: [Option<FighterFrameBoxes>; 2] = [None, None];
        let mut command_data_errors = Vec::new();
        for fighter_index in 0..2 {
            let character_id = state.fighters[fighter_index].character_id;
            if character_id < 16 {
                match embedded_character_command_rules(character_id) {
                    Ok(rules) => command_rules[fighter_index] = Some(rules),
                    Err(error) => command_data_errors.push(error.to_string()),
                }
                match load_fighter_frame_boxes(character_id) {
                    Ok(tables) => frame_boxes[fighter_index] = Some(tables),
                    Err(error) => command_data_errors
                        .push(format!("fighter {fighter_index} frame tables: {error}")),
                }
            }
        }
        Self {
            state,
            config,
            rng: Fpc322Rng::new(rng_seed),
            pending_specials: Vec::new(),
            command_rules,
            command_recognizers: [CommandRecognizer::default(); 2],
            command_data_errors,
            frame_boxes,
            custom_character_registry: CustomCharacterRegistry::default(),
            custom_character_definitions: BTreeMap::new(),
            last_report: FightTickReport::default(),
        }
    }

    /// Install and activate a validated data-only custom fighter package.
    ///
    /// The package cannot execute code. Its declarative frame/move tables are
    /// interpreted by this runtime and can be used by native and WASM hosts.
    pub fn register_custom_character_package(
        &mut self,
        package: CustomCharacterPackage,
    ) -> Result<u8, String> {
        let definition = CustomCharacterDefinition::from_package(&package)?;
        if self.custom_character_definitions.contains_key(&definition.runtime_id) {
            return Err(format!(
                "custom runtime ID {} is already registered",
                definition.runtime_id
            ));
        }
        let package_id = definition.package_id.clone();
        let runtime_id = definition.runtime_id;
        self.custom_character_registry
            .install(package)
            .map_err(|error| error.to_string())?;
        self.custom_character_registry
            .enable(&package_id)
            .map_err(|error| error.to_string())?;
        for fighter_index in 0..2 {
            if self.state.fighters[fighter_index].character_id == runtime_id {
                self.frame_boxes[fighter_index] = Some(definition.frame_boxes.clone());
                self.state.fighters[fighter_index].speed = definition.walk_speed;
            }
        }
        self.custom_character_definitions
            .insert(runtime_id, definition);
        Ok(runtime_id)
    }

    /// Select an installed custom fighter for one of the two fight slots.
    pub fn select_custom_character(
        &mut self,
        fighter_index: usize,
        package_id: &str,
    ) -> Result<u8, String> {
        if fighter_index >= 2 {
            return Err("fighter index must be 0 or 1".to_owned());
        }
        let package = self
            .custom_character_registry
            .get(package_id)
            .ok_or_else(|| format!("custom character {package_id:?} is not installed"))?;
        let runtime_id = self
            .custom_character_definitions
            .values()
            .find(|definition| definition.package_id == package.manifest.id)
            .map(|definition| definition.runtime_id)
            .ok_or_else(|| format!("custom character {package_id:?} has no runtime definition"))?;
        let definition = self
            .custom_character_definitions
            .get(&runtime_id)
            .cloned()
            .ok_or_else(|| "custom character definition disappeared".to_owned())?;

        self.state.fighters[fighter_index].character_id = runtime_id;
        self.command_rules[fighter_index] = None;
        self.command_recognizers[fighter_index] = CommandRecognizer::default();
        self.state.fighters[fighter_index].action_id = ACTION_IDLE;
        self.state.fighters[fighter_index].action_frame = 0;
        self.state.fighters[fighter_index].action_duration = 99;
        self.state.fighters[fighter_index].sprite_frame =
            (fighter_index as i32) * 50 + 1;
        self.state.fighters[fighter_index].speed = definition.walk_speed;
        self.state.fighters[fighter_index].attack_active = false;
        self.state.fighters[fighter_index].attack_box = None;
        self.state.fighters[fighter_index].invulnerable = false;
        self.frame_boxes[fighter_index] = Some(definition.frame_boxes);
        Ok(runtime_id)
    }

    /// Start a named special move using its validated weak/strong timeline.
    /// Hosts on both native and WASM call this same engine API after sampling input.
    pub fn queue_custom_character_move(
        &mut self,
        fighter_index: usize,
        move_id: &str,
        force_strength: u8,
    ) -> Result<(), String> {
        if fighter_index >= 2 {
            return Err("fighter index must be 0 or 1".to_owned());
        }
        if !matches!(force_strength, 1 | 2) {
            return Err("custom move force must be weak (1) or strong (2)".to_owned());
        }
        let runtime_id = self.state.fighters[fighter_index].character_id;
        let definition = self
            .custom_character_definitions
            .get(&runtime_id)
            .cloned()
            .ok_or_else(|| format!("fighter {fighter_index} is not a registered custom fighter"))?;
        let movement = definition
            .move_by_id(move_id)
            .cloned()
            .ok_or_else(|| format!("custom move {move_id:?} does not exist"))?;

        let current_action = self.state.fighters[fighter_index].action_id;
        let allowed = match movement.action_id {
            ACTION_SPECIAL1 | ACTION_SPECIAL2 | ACTION_SPECIAL4 => matches!(
                current_action,
                ACTION_IDLE | ACTION_WALKING | ACTION_DEFENDING | ACTION_LOW_DEFENDING
            ),
            ACTION_SPECIAL3 => matches!(
                current_action,
                ACTION_IDLE | ACTION_WALKING | ACTION_JUMPING | ACTION_LOW_DEFENDING | ACTION_SPECIAL2
            ),
            ACTION_SPECIAL_FINAL => matches!(
                current_action,
                ACTION_IDLE
                    | ACTION_WALKING
                    | ACTION_STRONG_PUNCH
                    | ACTION_CROUCHING
                    | ACTION_DEFENDING
                    | ACTION_LOW_DEFENDING
            ),
            _ => false,
        };
        if !allowed {
            return Err(format!(
                "custom move {:?} cannot start from action {current_action}",
                movement.id
            ));
        }
        if movement.requires_low_health_or_power
            && self.state.fighters[fighter_index].health > 15.0
            && self.state.fighters[fighter_index].power == 0
        {
            return Err("final special requires health <= 15 or available power".to_owned());
        }
        if movement.action_id == ACTION_SPECIAL3
            && self.config.special.ground_y - 90.0 > self.state.fighters[fighter_index].y
        {
            return Err("Chipote Voador can only start close to the ground".to_owned());
        }

        let duration = if force_strength == 1 {
            movement.duration_weak
        } else {
            movement.duration_strong
        };
        let sprite_frame = if force_strength == 1 {
            movement.sprite_frames_weak[0]
        } else {
            movement.sprite_frames_strong[0]
        };
        let fighter = &mut self.state.fighters[fighter_index];
        if movement.charge_multiplier > 0.0
            && fighter.charge_power < self.config.special.charge_max
        {
            fighter.charge_power += movement.charge_multiplier * self.config.special.charge_gain;
        }
        if movement.requires_low_health_or_power {
            if fighter.power == 3 {
                fighter.charge_power = 0.0;
            }
            fighter.power = fighter.power.wrapping_sub(1);
        }
        if movement.action_id == ACTION_SPECIAL3 {
            fighter.y = self.config.special.ground_y;
            fighter.in_air = false;
            fighter.vertical_velocity = 0;
            fighter.jump_direction = 0;
            fighter.low_time = 0;
            fighter.sequence_state = 0;
        }
        fighter.action_id = movement.action_id;
        fighter.action_frame = 0;
        fighter.action_duration = duration;
        fighter.force_strength = if movement.action_id == ACTION_SPECIAL_FINAL {
            3
        } else {
            force_strength
        };
        fighter.sprite_frame = (fighter_index as i32) * 50 + i32::from(sprite_frame);
        fighter.defending = false;
        fighter.attack_active = false;
        fighter.attack_box = None;
        fighter.is_kick = movement.is_kick;
        Ok(())
    }

    /// Read the validated definition needed by a native or Web/WASM renderer.
    /// Includes source-derived centre offsets, collision tables and logical move names.
    pub fn custom_character_definition(
        &self,
        runtime_id: u8,
    ) -> Option<&CustomCharacterDefinition> {
        self.custom_character_definitions.get(&runtime_id)
    }

    /// Retrieve a package asset by the fighter's runtime ID and logical asset ID.
    /// Native renderers and browser hosts use the same lookup after selecting a fighter.
    pub fn custom_character_asset(
        &self,
        runtime_id: u8,
        asset_id: &str,
    ) -> Option<&[u8]> {
        let definition = self.custom_character_definitions.get(&runtime_id)?;
        self.custom_character_registry
            .get(&definition.package_id)?
            .asset_bytes(asset_id)
    }

    /// Queue an explicit C01 command for the next non-stopped simulation tick.
    pub fn queue_chaves_special(
        &mut self,
        fighter_index: usize,
        command: ChavesSpecialCommand,
    ) -> bool {
        if fighter_index >= self.state.fighters.len()
            || self.state.fighters[fighter_index].character_id != 1
        {
            return false;
        }
        self.pending_specials.push((fighter_index, command));
        true
    }

    pub fn last_report(&self) -> &FightTickReport {
        &self.last_report
    }

    pub fn rng_calls_consumed(&self) -> u64 {
        self.rng.calls_consumed()
    }

    /// Run with independent normalized input frames for both fighters.
    /// The SimulationTick trait currently supplies player one and neutral player two.
    pub fn tick_with_inputs(&mut self, inputs: [InputFrame; 2]) {
        self.run_tick(inputs);
    }

    fn dispatch_character_command(
        &mut self,
        fighter_index: usize,
        button: AttackButton,
        command_code: u8,
        report: &mut FightTickReport,
    ) {
        report.command_matches.push(CommandMatch {
            fighter_index,
            button,
            command_code,
        });

        if self.state.fighters[fighter_index].character_id == 8 && matches!(command_code, 14..=24) {
            match self.execute_character_rot_action(fighter_index, 8, command_code) {
                Ok(()) => report
                    .rot_action_executions
                    .push((fighter_index, 8, command_code)),
                Err(error) => report.errors.push(format!(
                    "C08 [Acao] fighter {fighter_index}, command {command_code}: {error}"
                )),
            }
        } else if self.state.fighters[fighter_index].character_id == 7
            && matches!(command_code, 14..=24)
        {
            match self.execute_character_rot_action(fighter_index, 7, command_code) {
                Ok(()) => report
                    .rot_action_executions
                    .push((fighter_index, 7, command_code)),
                Err(error) => report.errors.push(format!(
                    "C07 [Acao] fighter {fighter_index}, command {command_code}: {error}"
                )),
            }
        } else if self.state.fighters[fighter_index].character_id == 6
            && matches!(command_code, 14..=24)
        {
            match self.execute_character_rot_action(fighter_index, 6, command_code) {
                Ok(()) => report
                    .rot_action_executions
                    .push((fighter_index, 6, command_code)),
                Err(error) => report.errors.push(format!(
                    "C06 [Acao] fighter {fighter_index}, command {command_code}: {error}"
                )),
            }
        } else if self.state.fighters[fighter_index].character_id == 5
            && matches!(command_code, 14..=24)
        {
            match self.execute_character_rot_action(fighter_index, 5, command_code) {
                Ok(()) => report
                    .rot_action_executions
                    .push((fighter_index, 5, command_code)),
                Err(error) => report.errors.push(format!(
                    "C05 [Acao] fighter {fighter_index}, command {command_code}: {error}"
                )),
            }
        } else if self.state.fighters[fighter_index].character_id == 3
            && matches!(command_code, 16..=24)
        {
            match self.execute_character_rot_action(fighter_index, 3, command_code) {
                Ok(()) => report
                    .rot_action_executions
                    .push((fighter_index, 3, command_code)),
                Err(error) => report.errors.push(format!(
                    "C03 [Acao] fighter {fighter_index}, command {command_code}: {error}"
                )),
            }
        } else if self.state.fighters[fighter_index].character_id == 2
            && matches!(command_code, 16..=23)
        {
            match self.execute_character_rot_action(fighter_index, 2, command_code) {
                Ok(()) => report
                    .rot_action_executions
                    .push((fighter_index, 2, command_code)),
                Err(error) => report.errors.push(format!(
                    "C02 [Acao] fighter {fighter_index}, command {command_code}: {error}"
                )),
            }
        } else if self.state.fighters[fighter_index].character_id == 1
            && ChavesSpecialCommand::from_pascal_code(command_code).is_some()
        {
            match self.execute_character_rot_action(fighter_index, 1, command_code) {
                Ok(()) => report
                    .rot_action_executions
                    .push((fighter_index, 1, command_code)),
                Err(error) => report.errors.push(format!(
                    "C01 [Acao] fighter {fighter_index}, command {command_code}: {error}"
                )),
            }
        } else if self.state.fighters[fighter_index].character_id == 13
            && matches!(command_code, 14 | 16..=26)
        {
            match self.execute_character_rot_action(fighter_index, 13, command_code) {
                Ok(()) => report
                    .rot_action_executions
                    .push((fighter_index, 13, command_code)),
                Err(error) => report.errors.push(format!(
                    "C13 [Acao] fighter {fighter_index}, command {command_code}: {error}"
                )),
            }
        } else if self.state.fighters[fighter_index].character_id == 14
            && matches!(command_code, 14 | 16..=24)
        {
            match self.execute_character_rot_action(fighter_index, 14, command_code) {
                Ok(()) => report
                    .rot_action_executions
                    .push((fighter_index, 14, command_code)),
                Err(error) => report.errors.push(format!(
                    "C14 [Acao] fighter {fighter_index}, command {command_code}: {error}"
                )),
            }
        } else if self.state.fighters[fighter_index].character_id == 15
            && matches!(command_code, 14 | 16..=24)
        {
            match self.execute_character_rot_action(fighter_index, 15, command_code) {
                Ok(()) => report
                    .rot_action_executions
                    .push((fighter_index, 15, command_code)),
                Err(error) => report.errors.push(format!(
                    "C15 [Acao] fighter {fighter_index}, command {command_code}: {error}"
                )),
            }
        } else if self.state.fighters[fighter_index].character_id == 10
            && matches!(command_code, 14..=24)
        {
            match self.execute_character_rot_action(fighter_index, 10, command_code) {
                Ok(()) => report
                    .rot_action_executions
                    .push((fighter_index, 10, command_code)),
                Err(error) => report.errors.push(format!(
                    "C10 [Acao] fighter {fighter_index}, command {command_code}: {error}"
                )),
            }
        } else if self.state.fighters[fighter_index].character_id == 11
            && matches!(command_code, 14..=24)
        {
            match self.execute_character_rot_action(fighter_index, 11, command_code) {
                Ok(()) => report
                    .rot_action_executions
                    .push((fighter_index, 11, command_code)),
                Err(error) => report.errors.push(format!(
                    "C11 [Acao] fighter {fighter_index}, command {command_code}: {error}"
                )),
            }
        } else if self.state.fighters[fighter_index].character_id == 12
            && matches!(command_code, 14..=22)
        {
            match self.execute_character_rot_action(fighter_index, 12, command_code) {
                Ok(()) => report
                    .rot_action_executions
                    .push((fighter_index, 12, command_code)),
                Err(error) => report.errors.push(format!(
                    "C12 [Acao] fighter {fighter_index}, command {command_code}: {error}"
                )),
            }
        } else if self.state.fighters[fighter_index].character_id == 9
            && matches!(command_code, 14 | 15 | 20 | 21)
        {
            match self.execute_character_rot_action(fighter_index, 9, command_code) {
                Ok(()) => report
                    .rot_action_executions
                    .push((fighter_index, 9, command_code)),
                Err(error) => report.errors.push(format!(
                    "C09 [Acao] fighter {fighter_index}, command {command_code}: {error}"
                )),
            }
        } else if self.state.fighters[fighter_index].character_id == 4 && command_code == 24 {
            match self.execute_character_rot_action(fighter_index, 4, command_code) {
                Ok(()) => report
                    .rot_action_executions
                    .push((fighter_index, 4, command_code)),
                Err(error) => report.errors.push(format!(
                    "C04 [Acao] fighter {fighter_index}, command {command_code}: {error}"
                )),
            }
        } else {
            // Even a rejected/unimplemented character command still executes
            // Common [Acao] and Common [AcaoFim] in Pascal's AcaoJogador path.
            self.state.fighters[fighter_index].defending = false;
            report.unhandled_commands.push(UnhandledCommand {
                fighter_index,
                button,
                command_code,
                source: UnhandledCommandSource::CharacterButtonRule,
            });
            let _ = self.state.finish_common_action(fighter_index);
        }
    }

    /// Execute one audited character [Acao] between the shared action sections.
    ///
    /// The common prelude, character script and common epilogue all share the
    /// same live state, RNG stream and helper bridge, while each ROT invocation
    /// receives its own Pascal local frame.
    fn execute_character_rot_action(
        &mut self,
        fighter_index: usize,
        character_id: u8,
        command_code: u8,
    ) -> Result<(), String> {
        if character_id >= 16 {
            let common = RotDocument::from_str(
                "Comum.rot",
                include_str!("../research/pascal-rot/Comum.rot"),
            )
            .map_err(|error| error.to_string())?;
            let jog = (fighter_index + 1) as u8;
            let special = self.config.special;
            let mut host = RotCombatHost {
                state: &mut self.state,
                rng: &mut self.rng,
                player_index: fighter_index,
                charge_gain: special.charge_gain,
                charge_max: special.charge_max,
                ground_y: special.ground_y,
                common: &common,
                recursion_depth: 0,
            };
            execute_rot_section(&common, "Acao", jog, command_code, &mut host)?;
            execute_rot_section(&common, "AcaoFim", jog, command_code, &mut host)?;
            return Ok(());
        }
        let (filename, source) = match character_id {
            1 => ("C01.rot", include_str!("../research/pascal-rot/C01.rot")),
            2 => ("C02.rot", include_str!("../research/pascal-rot/C02.rot")),
            3 => ("C03.rot", include_str!("../research/pascal-rot/C03.rot")),
            4 => ("C04.rot", include_str!("../research/pascal-rot/C04.rot")),
            5 => ("C05.rot", include_str!("../research/pascal-rot/C05.rot")),
            6 => ("C06.rot", include_str!("../research/pascal-rot/C06.rot")),
            7 => ("C07.rot", include_str!("../research/pascal-rot/C07.rot")),
            8 => ("C08.rot", include_str!("../research/pascal-rot/C08.rot")),
            9 => ("C09.rot", include_str!("../research/pascal-rot/C09.rot")),
            10 => ("C10.rot", include_str!("../research/pascal-rot/C10.rot")),
            11 => ("C11.rot", include_str!("../research/pascal-rot/C11.rot")),
            12 => ("C12.rot", include_str!("../research/pascal-rot/C12.rot")),
            13 => ("C13.rot", include_str!("../research/pascal-rot/C13.rot")),
            14 => ("C14.rot", include_str!("../research/pascal-rot/C14.rot")),
            15 => ("C15.rot", include_str!("../research/pascal-rot/C15.rot")),
            _ => {
                return Err(format!(
                    "character C{character_id:02} has no live ROT route"
                ))
            }
        };
        let common = RotDocument::from_str(
            "Comum.rot",
            include_str!("../research/pascal-rot/Comum.rot"),
        )
        .map_err(|error| error.to_string())?;
        let character =
            RotDocument::from_str(filename, source).map_err(|error| error.to_string())?;

        let jog = (fighter_index + 1) as u8;
        let special = self.config.special;
        let mut host = RotCombatHost {
            state: &mut self.state,
            rng: &mut self.rng,
            player_index: fighter_index,
            charge_gain: special.charge_gain,
            charge_max: special.charge_max,
            ground_y: special.ground_y,
            common: &common,
            recursion_depth: 0,
        };

        execute_rot_section(&common, "Acao", jog, command_code, &mut host)?;
        execute_rot_section(&character, "Acao", jog, command_code, &mut host)?;
        execute_rot_section(&common, "AcaoFim", jog, command_code, &mut host)?;
        Ok(())
    }

    /// Retained C13 hand-port used only as a differential test oracle.
    ///
    /// Production dispatch uses C13.rot through the shared ROT host. Keep this
    /// implementation until Pascal/ROT differential coverage proves the migration.
    fn dispatch_c13_extra_compatibility_oracle(
        &mut self,
        command_code: u8,
        fighter_index: usize,
        report: &mut FightTickReport,
    ) {
        let fighter = &mut self.state.fighters[fighter_index];
        let sprite_offset = (fighter_index as i32) * 50;
        match command_code {
            25 => {
                if !matches!(
                    fighter.action_id,
                    ACTION_IDLE
                        | ACTION_WALKING
                        | ACTION_JUMPING
                        | ACTION_DEFENDING
                        | ACTION_LOW_DEFENDING
                ) {
                    return;
                }
                fighter.action_id = ACTION_SPECIAL4;
                fighter.action_frame = 0;
                fighter.action_duration = 30;
                if fighter.charge_power < self.config.special.charge_max {
                    fighter.charge_power += 5.0 * self.config.special.charge_gain;
                }
                fighter.force_strength = 4;
            }
            26 => {
                if !matches!(
                    fighter.action_id,
                    ACTION_IDLE
                        | ACTION_WALKING
                        | ACTION_CROUCHING
                        | ACTION_JUMPING
                        | ACTION_DEFENDING
                        | ACTION_LOW_DEFENDING
                        | ACTION_SPECIAL3
                ) || !(fighter.health <= 15.0 || fighter.power > 0)
                {
                    return;
                }
                fighter.action_id = ACTION_SPECIAL_FINAL;
                fighter.action_frame = 0;
                fighter.action_duration = 30;
                fighter.in_air = false;
                fighter.jump_direction = 0;
                fighter.vertical_velocity = 0;
                if fighter.power == 3 {
                    fighter.charge_power = 0.0;
                }
                fighter.power = fighter.power.wrapping_sub(1);
                fighter.sprite_frame = sprite_offset + 37;
                self.state.screen_flash = 22;
                self.state.pending_sound = Some(crate::chaves_specials::SOUND_FINAL_SPECIAL);
            }
            _ => return,
        }
        let _ = report;
    }

    /// Shared Pascal [Acao] for commands 3..6, followed by [AcaoFim].
    fn dispatch_common_basic_action(
        &mut self,
        fighter_index: usize,
        button: AttackButton,
        command_code: u8,
        report: &mut FightTickReport,
    ) {
        report.command_matches.push(CommandMatch {
            fighter_index,
            button,
            command_code,
        });
        let context = self.config.special;
        let fighter = &mut self.state.fighters[fighter_index];
        fighter.defending = false;
        let action = fighter.action_id;
        let grounded = fighter.vertical_velocity == 0;
        let mut next = None;
        let mut duration = 0;
        let mut charge = 0.0;
        let mut sound = None;

        match command_code {
            3 => {
                if grounded && matches!(action, ACTION_IDLE | ACTION_WALKING | ACTION_DEFENDING) {
                    next = Some(ACTION_WEAK_PUNCH);
                    duration = 6;
                    charge = 1.0;
                    sound = Some(5);
                } else if matches!(action, ACTION_CROUCHING | ACTION_LOW_DEFENDING) {
                    next = Some(ACTION_CROUCH_WEAK_PUNCH);
                    duration = 6;
                    charge = 1.0;
                    sound = Some(5);
                } else if action == ACTION_JUMPING {
                    next = Some(ACTION_AIR_WEAK_PUNCH);
                    duration = 15;
                    charge = 1.0;
                    sound = Some(5);
                }
            }
            4 => {
                if grounded
                    && (matches!(action, ACTION_IDLE | ACTION_WALKING | ACTION_DEFENDING)
                        || (action == ACTION_WEAK_PUNCH && fighter.action_frame >= 2))
                {
                    next = Some(ACTION_WEAK_KICK);
                    duration = 6;
                    charge = 1.0;
                    sound = Some(5);
                } else if matches!(action, ACTION_CROUCHING | ACTION_LOW_DEFENDING)
                    || (action == ACTION_CROUCH_WEAK_PUNCH && fighter.action_frame >= 2)
                {
                    next = Some(ACTION_CROUCH_WEAK_KICK);
                    duration = 6;
                    charge = 1.0;
                    sound = Some(5);
                } else if action == ACTION_JUMPING {
                    next = Some(ACTION_AIR_WEAK_KICK);
                    duration = 15;
                    charge = 1.0;
                    sound = Some(5);
                }
            }
            5 => {
                if grounded
                    && (matches!(action, ACTION_IDLE | ACTION_WALKING | ACTION_DEFENDING)
                        || (matches!(action, ACTION_WEAK_PUNCH | ACTION_WEAK_KICK)
                            && fighter.action_frame >= 2))
                {
                    next = Some(ACTION_STRONG_PUNCH);
                    duration = 11;
                    charge = 2.0;
                    sound = Some(6);
                } else if matches!(action, ACTION_CROUCHING | ACTION_LOW_DEFENDING)
                    || (matches!(action, ACTION_CROUCH_WEAK_PUNCH | ACTION_CROUCH_WEAK_KICK)
                        && fighter.action_frame >= 2)
                {
                    next = Some(ACTION_CROUCH_STRONG_PUNCH);
                    duration = 11;
                    charge = 2.0;
                    sound = Some(6);
                } else if action == ACTION_JUMPING {
                    next = Some(ACTION_AIR_STRONG_PUNCH);
                    duration = 11;
                    charge = 2.0;
                    sound = Some(6);
                }
            }
            6 => {
                if grounded
                    && !fighter.in_air
                    && matches!(action, ACTION_IDLE | ACTION_WALKING | ACTION_DEFENDING)
                {
                    next = Some(ACTION_STRONG_KICK);
                    duration = 11;
                    charge = 2.0;
                    sound = Some(6);
                } else if matches!(action, ACTION_CROUCHING | ACTION_LOW_DEFENDING)
                    || (action == ACTION_CROUCH_WEAK_KICK && fighter.action_frame >= 2)
                {
                    next = Some(ACTION_CROUCH_STRONG_KICK);
                    duration = if fighter.character_id == 13 { 18 } else { 11 };
                    charge = 2.0;
                    fighter.is_kick = true;
                    sound = Some(6);
                } else if action == ACTION_JUMPING {
                    next = Some(ACTION_AIR_STRONG_KICK);
                    duration = 11;
                    charge = 2.0;
                    sound = Some(6);
                }
            }
            _ => {}
        }

        if let Some(action_id) = next {
            fighter.action_id = action_id;
            fighter.action_frame = 0;
            fighter.action_duration = duration;
            if fighter.charge_power < context.charge_max {
                fighter.charge_power += charge * context.charge_gain;
            }
            if let Some(sound_id) = sound {
                self.state.pending_sound = Some(sound_id);
            }
        }
        let _ = self.state.finish_common_action(fighter_index);
    }

    /// Apply the directional portion of Pascal L7_Entrada before attack buttons.
    fn dispatch_directional_input(&mut self, fighter_index: usize, input: InputFrame) {
        let direction = input.direction;
        let down = matches!(
            direction,
            Direction::Down | Direction::DownLeft | Direction::DownRight
        );
        let horizontal = match direction {
            Direction::Left | Direction::UpLeft | Direction::DownLeft => Some(-1.0),
            Direction::Right | Direction::UpRight | Direction::DownRight => Some(1.0),
            _ => None,
        };

        // L7_Entrada calls AcaoJogador for each horizontal/down command even
        // when the Common [Acao] guard rejects the requested transition.
        let finish_common = {
            let fighter = &mut self.state.fighters[fighter_index];
            let mut finish_common = false;

            if !down {
                if horizontal.is_some() {
                    // Common [Acao] clears defending even when its movement guard rejects.
                    fighter.defending = false;
                }
                if let Some(sign) = horizontal {
                    finish_common = true;
                    if matches!(fighter.action_id, ACTION_IDLE | ACTION_WALKING) {
                        fighter.x += sign * fighter.speed;
                        fighter.action_id = ACTION_WALKING;
                        fighter.action_duration = 99;
                    }
                }
            } else {
                finish_common = true;
                // The shared action prelude runs before the crouch eligibility check.
                fighter.defending = false;
                if matches!(fighter.action_id, ACTION_IDLE | ACTION_WALKING) {
                    fighter.action_id = ACTION_CROUCHING;
                    fighter.action_duration = 99;
                }
            }

            if !down && fighter.action_id == ACTION_CROUCHING {
                fighter.action_id = ACTION_IDLE;
            }
            if horizontal.is_none() && !down && fighter.action_id == ACTION_WALKING {
                fighter.action_id = ACTION_IDLE;
                fighter.action_frame = 0;
            }
            finish_common
        };

        if finish_common {
            let _ = self.state.finish_common_action(fighter_index);
        }

        if matches!(
            direction,
            Direction::Up | Direction::UpLeft | Direction::UpRight
        ) {
            let fighter = &mut self.state.fighters[fighter_index];
            if matches!(
                fighter.action_id,
                ACTION_IDLE | ACTION_WALKING | ACTION_DEFENDING | ACTION_LOW_DEFENDING
            ) && fighter.vertical_velocity == 0
                && !fighter.in_air
            {
                fighter.vertical_velocity = -32;
                fighter.in_air = true;
                fighter.action_id = ACTION_JUMPING;
                fighter.action_frame = 0;
                fighter.action_duration = 99;
                fighter.defending = false;
                fighter.jump_direction = match direction {
                    Direction::UpLeft => 2,
                    Direction::UpRight => 3,
                    _ => 1,
                };
            }
        }
    }

    fn dispatch_input_commands(&mut self, inputs: [InputFrame; 2], report: &mut FightTickReport) {
        // L7_Entrada processes player one, then player two. Directional commands
        // precede button edges exactly as in the Pascal input routine.
        for fighter_index in 0..2 {
            let input = inputs[fighter_index];
            self.dispatch_directional_input(fighter_index, input);
            let Some(button_rules) = self.command_rules[fighter_index]
                .as_ref()
                .map(|rules| rules.buttons.clone())
            else {
                continue;
            };
            for button in [
                AttackButton::WeakPunch,
                AttackButton::WeakKick,
                AttackButton::StrongPunch,
                AttackButton::StrongKick,
            ] {
                let commands = self.command_recognizers[fighter_index].commands_for_button(
                    &self.state.fighters[fighter_index],
                    input,
                    button,
                    &button_rules,
                    self.config.special.ground_y,
                );
                for command_code in commands {
                    self.dispatch_character_command(fighter_index, button, command_code, report);
                }
            }

            // L7_Entrada calls golpe() first, then AcaoJogador() for the same edge.
            for (button, command_code) in [
                (AttackButton::WeakPunch, 3),
                (AttackButton::WeakKick, 4),
                (AttackButton::StrongPunch, 5),
                (AttackButton::StrongKick, 6),
            ] {
                if input.was_pressed(button.input_button()) {
                    self.dispatch_common_basic_action(fighter_index, button, command_code, report);
                }
            }
        }
    }

    fn update_command_sequences(&mut self, inputs: [InputFrame; 2], report: &mut FightTickReport) {
        for fighter_index in 0..2 {
            let input = inputs[fighter_index];
            let Some(rules) = self.command_rules[fighter_index].as_ref() else {
                continue;
            };
            let recognizer = &mut self.command_recognizers[fighter_index];
            recognizer.update_counters(&mut self.state.fighters[fighter_index], input);
            let transitions = recognizer.apply_sequences(
                &mut self.state.fighters[fighter_index],
                input,
                &rules.sequences,
                self.config.special.ground_y,
            );
            report.sequence_transitions.extend(
                transitions
                    .into_iter()
                    .map(|(from, to)| (fighter_index, from, to)),
            );
            recognizer.tick_sequence_timeout(
                &mut self.state.fighters[fighter_index],
                self.state.screen_flash != 0 || self.state.clock_stopped,
            );
        }
    }

    /// Run one Pascal AtualizaQuadro update in source order.
    fn advance_fighter_frame(&mut self, fighter_index: usize, report: &mut FightTickReport) {
        if self.state.clock_stopped || self.state.screen_flash != 0 {
            return;
        }

        let character_id = self.state.fighters[fighter_index].character_id;
        let action_id = self.state.fighters[fighter_index].action_id;
        let frame_before = self.state.fighters[fighter_index].action_frame;
        let jog = (fighter_index + 1) as u8;
        let special = self.config.special;

        let common = match RotDocument::from_str(
            "Comum.rot",
            include_str!("../research/pascal-rot/Comum.rot"),
        ) {
            Ok(document) => document,
            Err(error) => {
                report.errors.push(format!("Comum.rot [Quadro]: {error}"));
                return;
            }
        };

        // Pascal executes Common [Quadro] first, using the fight's shared state/RNG.
        let common_result = {
            let mut host = RotCombatHost {
                state: &mut self.state,
                rng: &mut self.rng,
                player_index: fighter_index,
                charge_gain: special.charge_gain,
                charge_max: special.charge_max,
                ground_y: special.ground_y,
                common: &common,
                recursion_depth: 0,
            };
            execute_rot_section(&common, "Quadro", jog, 0, &mut host)
        };
        if let Err(error) = common_result {
            report.errors.push(format!(
                "Comum.rot [Quadro] fighter {fighter_index}: {error}"
            ));
            return;
        }

        if self.custom_character_definitions.contains_key(&character_id) {
            if let Err(error) = self.advance_custom_character_frame(fighter_index, report) {
                report.errors.push(format!(
                    "custom character {character_id} [Quadro] fighter {fighter_index}: {error}"
                ));
            }
            return;
        }

        // Preserve the existing, tested C01 manual frame path for its implemented
        // special/victory/pose branches. Other branches and C02-C15 use live ROT.
        let c01_manual = character_id == 1
            && matches!(
                action_id,
                ACTION_SPECIAL1
                    | ACTION_SPECIAL2
                    | ACTION_SPECIAL3
                    | ACTION_SPECIAL4
                    | ACTION_SPECIAL_FINAL
                    | ACTION_VICTORY
                    | ACTION_POSE
            );

        if c01_manual {
            let previous_errors = report.errors.len();
            self.advance_c01_frame(fighter_index, report);
            if report.errors.len() != previous_errors {
                return;
            }
        } else {
            let (filename, source) = match character_id {
                1 => ("C01.rot", include_str!("../research/pascal-rot/C01.rot")),
                2 => ("C02.rot", include_str!("../research/pascal-rot/C02.rot")),
                3 => ("C03.rot", include_str!("../research/pascal-rot/C03.rot")),
                4 => ("C04.rot", include_str!("../research/pascal-rot/C04.rot")),
                5 => ("C05.rot", include_str!("../research/pascal-rot/C05.rot")),
                6 => ("C06.rot", include_str!("../research/pascal-rot/C06.rot")),
                7 => ("C07.rot", include_str!("../research/pascal-rot/C07.rot")),
                8 => ("C08.rot", include_str!("../research/pascal-rot/C08.rot")),
                9 => ("C09.rot", include_str!("../research/pascal-rot/C09.rot")),
                10 => ("C10.rot", include_str!("../research/pascal-rot/C10.rot")),
                11 => ("C11.rot", include_str!("../research/pascal-rot/C11.rot")),
                12 => ("C12.rot", include_str!("../research/pascal-rot/C12.rot")),
                13 => ("C13.rot", include_str!("../research/pascal-rot/C13.rot")),
                14 => ("C14.rot", include_str!("../research/pascal-rot/C14.rot")),
                15 => ("C15.rot", include_str!("../research/pascal-rot/C15.rot")),
                _ => {
                    report.errors.push(format!(
                        "fighter {fighter_index} has no pinned [Quadro] script for character {character_id}"
                    ));
                    return;
                }
            };
            let character = match RotDocument::from_str(filename, source) {
                Ok(document) => document,
                Err(error) => {
                    report.errors.push(format!("{filename} [Quadro]: {error}"));
                    return;
                }
            };
            let character_result = {
                let mut host = RotCombatHost {
                    state: &mut self.state,
                    rng: &mut self.rng,
                    player_index: fighter_index,
                    charge_gain: special.charge_gain,
                    charge_max: special.charge_max,
                    ground_y: special.ground_y,
                    common: &common,
                    recursion_depth: 0,
                };
                execute_rot_section(&character, "Quadro", jog, 0, &mut host)
            };
            if let Err(error) = character_result {
                report.errors.push(format!(
                    "{filename} [Quadro] fighter {fighter_index}: {error}"
                ));
                return;
            }
            report
                .rot_frame_executions
                .push((fighter_index, character_id));
        }

        // AplicarCaixas runs only after both frame sections completed successfully.
        if let Err(error) = self.apply_fighter_boxes(fighter_index) {
            report
                .errors
                .push(format!("AplicarCaixas fighter {fighter_index}: {error}"));
            return;
        }
        report.box_updates.push(fighter_index);

        // AtualizaQuadro/AplicarCaixas observe the same action timer. Advance
        // only after the selected boxes have been stored for this frame.
        let fighter = &mut self.state.fighters[fighter_index];
        if fighter.action_frame == frame_before
            && fighter.action_duration != 99
            && fighter.action_frame < fighter.action_duration
        {
            fighter.action_frame = fighter.action_frame.saturating_add(1);
        }
    }

    /// Advance a custom character's declarative frame/move timeline.
    fn advance_custom_character_frame(
        &mut self,
        fighter_index: usize,
        report: &mut FightTickReport,
    ) -> Result<(), String> {
        let runtime_id = self.state.fighters[fighter_index].character_id;
        let definition = self
            .custom_character_definitions
            .get(&runtime_id)
            .cloned()
            .ok_or_else(|| format!("custom runtime ID {runtime_id} is not registered"))?;
        let action_id = self.state.fighters[fighter_index].action_id;
        let action_frame = self.state.fighters[fighter_index].action_frame;
        let force_strength = self.state.fighters[fighter_index].force_strength;
        let facing = self.state.fighters[fighter_index].facing;
        let offset = (fighter_index as i32) * 50;

        self.state.fighters[fighter_index].speed = definition.walk_speed;
        if let Some(movement) = definition.move_by_action(action_id).cloned() {
            let timeline = if force_strength == 1 {
                &movement.sprite_frames_weak
            } else {
                &movement.sprite_frames_strong
            };
            let sprite = *timeline
                .get(usize::from(action_frame))
                .ok_or_else(|| format!("move {:?} has no sprite for frame {action_frame}", movement.id))?;
            {
                let fighter = &mut self.state.fighters[fighter_index];
                fighter.sprite_frame = offset + i32::from(sprite);
                fighter.is_kick = movement.is_kick;
                if let Some(last_invulnerable_frame) = movement.self_invulnerable_until {
                    fighter.invulnerable = action_frame <= last_invulnerable_frame;
                }
                if movement.vertical_delta != 0.0 {
                    fighter.y += movement.vertical_delta;
                }
                for window in &movement.push_windows {
                    if (window.start_frame..=window.end_frame).contains(&action_frame) {
                        let distance = if force_strength == 1 {
                            window.weak_distance
                        } else {
                            window.strong_distance
                        };
                        fighter.x += if facing == SIDE_LEFT { -distance } else { distance };
                    }
                }
            }

            let opponent_index = 1 - fighter_index;
            let opponent_hits = self.state.fighters[fighter_index].hit_count;
            for rule in &movement.opponent_invulnerability {
                let hit_count_matches = opponent_hits >= rule.min_hit_count
                    && rule.max_hit_count.map_or(true, |maximum| opponent_hits <= maximum);
                if (rule.start_frame..=rule.end_frame).contains(&action_frame)
                    && hit_count_matches
                {
                    self.state.fighters[opponent_index].invulnerable = rule.value;
                }
            }

            for event in &movement.effects {
                if event.frame != action_frame {
                    continue;
                }
                let variant = if force_strength == 1 {
                    event.weak_variant
                } else {
                    event.strong_variant
                };
                let x_offset = if facing == SIDE_LEFT {
                    -i32::from(event.offset_x)
                } else {
                    i32::from(event.offset_x)
                };
                report.custom_character_events.push(CustomCharacterEvent::SpawnEffect {
                    fighter_index,
                    effect_id: event.effect_id,
                    variant,
                    x: self.state.fighters[fighter_index].x + f64::from(x_offset),
                    y: self.state.fighters[fighter_index].y + f64::from(event.offset_y),
                });
            }
            for event in &movement.sounds {
                if event.frame == action_frame {
                    self.state.pending_sound = Some(event.sound_id);
                }
            }
            if let Some((frame, duration)) = movement.screen_shake {
                if frame == action_frame {
                    self.state.screen_shake = duration;
                }
            }
        } else {
            // Common movement/basic-attack animation remains owned by Comum.rot.
            let fighter = &mut self.state.fighters[fighter_index];
            if fighter.sprite_frame <= 0 {
                fighter.sprite_frame = offset + 1;
            }
        }

        self.apply_fighter_boxes(fighter_index)?;
        report
            .custom_character_frame_executions
            .push((fighter_index, runtime_id));

        let fighter = &mut self.state.fighters[fighter_index];
        if fighter.action_frame < fighter.action_duration {
            fighter.action_frame = fighter.action_frame.saturating_add(1);
        }
        if fighter.action_frame >= fighter.action_duration
            && definition.move_by_action(action_id).is_some()
        {
            fighter.action_id = ACTION_IDLE;
            fighter.action_frame = 0;
            fighter.action_duration = 99;
            fighter.attack_active = false;
            fighter.attack_box = None;
            fighter.invulnerable = false;
            fighter.is_kick = false;
        }
        Ok(())
    }

    /// Apply the current character's Corpo row and all matching Ataque rows.
    /// A selector miss retains the previously stored box, as in Pascal.
    fn apply_fighter_boxes(&mut self, fighter_index: usize) -> Result<(), String> {
        let tables = self.frame_boxes[fighter_index].as_ref().ok_or_else(|| {
            format!("no source frame/box table loaded for fighter {fighter_index}")
        })?;
        let fighter = &mut self.state.fighters[fighter_index];

        // Pascal subtracts the second player's 50-frame strip offset only for Corpo.
        let body_frame = fighter.sprite_frame - if fighter_index == 1 { 50 } else { 0 };
        if (0..=74).contains(&body_frame) {
            if let Some(row) = tables
                .hurtboxes
                .iter()
                .find(|row| i32::from(row.frame) == body_frame)
            {
                let bounds = if fighter.facing == SIDE_LEFT {
                    row.bounds.mirrored_horizontally()
                } else {
                    row.bounds
                };
                fighter.body = crate::combat_runtime::BodyBox {
                    x1: f64::from(bounds.x1),
                    y1: f64::from(bounds.y1),
                    x2: f64::from(bounds.x2),
                    y2: f64::from(bounds.y2),
                };
            }
        }

        // Evaluate force conditions against current force in original INI order.
        let mut attack_state = PascalAttackState {
            current_force: fighter.force_strength,
            current_box: fighter.attack_box,
        };
        let applied = attack_state.apply_ordered_rows(
            ActionId(fighter.action_id),
            fighter.action_frame,
            &tables.attacks,
        );
        if applied > 0 {
            let mut selected = attack_state
                .current_box
                .ok_or_else(|| "attack selector reported a match without a box".to_owned())?;
            if fighter.facing == SIDE_LEFT {
                selected.bounds = selected.bounds.mirrored_horizontally();
            }
            fighter.force_strength = attack_state.current_force;
            fighter.attack_active = selected.active;
            fighter.attack_box = Some(selected);
        }

        Ok(())
    }

    /// Detect current melee contacts without applying damage or mutating fighter state.
    ///
    /// Sprite heights are explicit until the source AlturaSprite values are represented
    /// in FighterState. Contacts are returned in Pascal P1-then-P2 evaluation order.
    pub fn detect_melee_contacts(
        &self,
        sprite_heights: [i32; 2],
    ) -> Result<
        Vec<crate::melee_collision::MeleeCollision>,
        crate::melee_collision::CollisionGeometryError,
    > {
        let mut effective_heights = sprite_heights;
        for fighter_index in 0..2 {
            let character_id = self.state.fighters[fighter_index].character_id;
            if let Some(definition) = self.custom_character_definitions.get(&character_id) {
                effective_heights[fighter_index] = definition.sprite_height;
            }
        }
        crate::melee_collision::detect_fight_melee_contacts(
            [&self.state.fighters[0], &self.state.fighters[1]],
            effective_heights,
            self.state.clock_stopped,
            self.state.screen_flash != 0,
        )
    }

    /// Execute Pascal's ordered melee consequence pipeline for the currently stored boxes.
    ///
    /// Contacts are evaluated P1→P2 then P2→P1. The common guard/damage/combo
    /// mutations run first, followed by the defender's common [Acao] reaction,
    /// optional knockdown, and finally the attacker's character-specific [Acerto].
    /// The damage multiplier and intro-warning gate are explicit source globals.
    pub fn resolve_melee_contacts(
        &mut self,
        sprite_heights: [i32; 2],
        damage_level: f64,
        entry_warning: bool,
    ) -> Result<Vec<CombatResolution>, String> {
        let contacts = self
            .detect_melee_contacts(sprite_heights)
            .map_err(|error| error.to_string())?;
        let mut resolutions = Vec::with_capacity(contacts.len());

        for contact in contacts {
            let attacker_index = contact.attacker_index;
            let defender_index = contact.defender_index;
            let defender_character = self.state.fighters[defender_index].character_id;
            let result = resolve_melee_contact(
                &mut self.state.fighters,
                contact,
                CombatResolutionContext {
                    damage_level,
                    entry_warning,
                    clock_stopped: self.state.clock_stopped,
                    ground_y: self.config.special.ground_y,
                },
            )
            .map_err(|error| format!("melee resolution: {error:?}"))?;

            match result {
                CombatResolution::Blocked {
                    defender_knocked_out,
                    chip_damage,
                    ..
                } => {
                    // Pascal calls the knockdown action before testing the guard again.
                    // Common [Acao] clears defending, so a chip KO continues as a hit
                    // without applying chip damage a second time.
                    if defender_knocked_out {
                        self.execute_character_rot_action(defender_index, defender_character, 10)?;
                        let continued = resolve_melee_contact_after_guard_break(
                            &mut self.state.fighters,
                            contact,
                            CombatResolutionContext {
                                damage_level,
                                entry_warning,
                                clock_stopped: self.state.clock_stopped,
                                ground_y: self.config.special.ground_y,
                            },
                            chip_damage,
                        )
                        .map_err(|error| format!("chip-KO continuation: {error:?}"))?;
                        match continued {
                            CombatResolution::Hit {
                                defender_knocked_out,
                                knockdown_after_reaction,
                                defender_reaction,
                                ..
                            } => {
                                if defender_knocked_out {
                                    self.execute_character_rot_action(
                                        defender_index,
                                        defender_character,
                                        10,
                                    )?;
                                }
                                self.execute_character_rot_action(
                                    defender_index,
                                    defender_character,
                                    if defender_reaction == 18 { 9 } else { 8 },
                                )?;
                                if knockdown_after_reaction {
                                    self.execute_character_rot_action(
                                        defender_index,
                                        defender_character,
                                        10,
                                    )?;
                                }
                                self.state.fighters[defender_index].recoil = 4;
                                self.state.fighters[attacker_index].attack_active = false;
                                self.state.fighters[defender_index].attack_active = false;
                                self.execute_character_acerto(attacker_index)?;
                                resolutions.push(continued);
                                continue;
                            }
                            CombatResolution::Blocked { .. } => {
                                return Err(
                                    "chip-KO continuation unexpectedly remained blocked".to_owned()
                                );
                            }
                        }
                    }
                    self.state.fighters[attacker_index].attack_active = false;
                }
                CombatResolution::Hit {
                    defender_knocked_out,
                    knockdown_after_reaction,
                    defender_reaction,
                    ..
                } => {
                    // A KO first requests cmDerrubar. Pascal then runs the standard
                    // high/low reaction, and may request a second knockdown afterwards.
                    if defender_knocked_out {
                        self.execute_character_rot_action(defender_index, defender_character, 10)?;
                    }
                    self.execute_character_rot_action(
                        defender_index,
                        defender_character,
                        if defender_reaction == 18 { 9 } else { 8 },
                    )?;
                    if knockdown_after_reaction {
                        self.execute_character_rot_action(defender_index, defender_character, 10)?;
                    }
                    // L7_Acertos writes O.Recuo only after the reaction commands.
                    self.state.fighters[defender_index].recoil = 4;
                    // Both active flags are consumed immediately before ExecutarAcerto.
                    self.state.fighters[attacker_index].attack_active = false;
                    self.state.fighters[defender_index].attack_active = false;
                    self.execute_character_acerto(attacker_index)?;
                }
            }
            resolutions.push(result);
        }

        Ok(resolutions)
    }

    /// Run the actual pinned character [Acerto] section against the live fighters.
    /// Missing sections are valid source data and execute as no-ops.
    fn execute_character_acerto(&mut self, fighter_index: usize) -> Result<(), String> {
        let runtime_id = self.state.fighters[fighter_index].character_id;
        if let Some(definition) = self.custom_character_definitions.get(&runtime_id) {
            let action_id = self.state.fighters[fighter_index].action_id;
            let force_strength = self.state.fighters[fighter_index].force_strength;
            if let Some(recoil) = definition.opponent_recoil(action_id, force_strength) {
                self.state.fighters[1 - fighter_index].recoil = recoil;
            }
            return Ok(());
        }
        let (filename, source) = match runtime_id {
            1 => ("C01.rot", include_str!("../research/pascal-rot/C01.rot")),
            2 => ("C02.rot", include_str!("../research/pascal-rot/C02.rot")),
            3 => ("C03.rot", include_str!("../research/pascal-rot/C03.rot")),
            4 => ("C04.rot", include_str!("../research/pascal-rot/C04.rot")),
            5 => ("C05.rot", include_str!("../research/pascal-rot/C05.rot")),
            6 => ("C06.rot", include_str!("../research/pascal-rot/C06.rot")),
            7 => ("C07.rot", include_str!("../research/pascal-rot/C07.rot")),
            8 => ("C08.rot", include_str!("../research/pascal-rot/C08.rot")),
            9 => ("C09.rot", include_str!("../research/pascal-rot/C09.rot")),
            10 => ("C10.rot", include_str!("../research/pascal-rot/C10.rot")),
            11 => ("C11.rot", include_str!("../research/pascal-rot/C11.rot")),
            12 => ("C12.rot", include_str!("../research/pascal-rot/C12.rot")),
            13 => ("C13.rot", include_str!("../research/pascal-rot/C13.rot")),
            14 => ("C14.rot", include_str!("../research/pascal-rot/C14.rot")),
            15 => ("C15.rot", include_str!("../research/pascal-rot/C15.rot")),
            other => return Err(format!("no [Acerto] script for character {other}")),
        };
        let character = RotDocument::from_str(filename, source).map_err(|e| e.to_string())?;
        let common = RotDocument::from_str(
            "Comum.rot",
            include_str!("../research/pascal-rot/Comum.rot"),
        )
        .map_err(|e| e.to_string())?;
        let special = self.config.special;
        let jog = (fighter_index + 1) as u8;
        let mut host = RotCombatHost {
            state: &mut self.state,
            rng: &mut self.rng,
            player_index: fighter_index,
            charge_gain: special.charge_gain,
            charge_max: special.charge_max,
            ground_y: special.ground_y,
            common: &common,
            recursion_depth: 0,
        };
        execute_rot_section(&character, "Acerto", jog, 0, &mut host)
    }

    fn advance_c01_frame(&mut self, fighter_index: usize, report: &mut FightTickReport) {
        if self.state.fighters[fighter_index].character_id != 1 {
            return;
        }

        let action_id = self.state.fighters[fighter_index].action_id;
        let advances_frame = matches!(
            action_id,
            ACTION_SPECIAL1
                | ACTION_SPECIAL2
                | ACTION_SPECIAL3
                | ACTION_SPECIAL4
                | ACTION_SPECIAL_FINAL
                | ACTION_VICTORY
                | ACTION_POSE
        );
        if !advances_frame {
            return;
        }

        let mut context = self.config.frame;
        context.sprite_offset = (fighter_index as i32) * 50;
        context.camera_x = self.state.camera_x;
        match self
            .state
            .update_c01_character_frame(fighter_index, context, &mut self.rng)
        {
            Ok(frame_result) => report.c01_frames.push((fighter_index, frame_result)),
            Err(error) => report
                .errors
                .push(format!("C01 [Quadro] fighter {fighter_index}: {error}")),
        }
    }

    fn run_tick(&mut self, inputs: [InputFrame; 2]) {
        let mut report = FightTickReport {
            tick_index: self.state.tick_index,
            input: inputs[0],
            fighter_inputs: inputs,
            errors: self.command_data_errors.clone(),
            ..FightTickReport::default()
        };

        // L7_Entrada is gated by hit-stop and screen flash. Sequence tracking still
        // runs afterwards, matching sequenciagolpe's placement in the Pascal loop.
        if !self.state.clock_stopped && self.state.screen_flash == 0 {
            self.dispatch_input_commands(inputs, &mut report);
        }
        self.update_command_sequences(inputs, &mut report);

        if !self.state.clock_stopped {
            for (fighter_index, command) in std::mem::take(&mut self.pending_specials) {
                match self.state.dispatch_chaves_special(
                    fighter_index,
                    command,
                    self.config.special,
                ) {
                    Ok(outcome) => report.special_outcomes.push((fighter_index, outcome)),
                    Err(error) => report
                        .errors
                        .push(format!("C01 [Acao] fighter {fighter_index}: {error}")),
                }
            }

            for fighter_index in 0..self.state.fighters.len() {
                self.advance_fighter_frame(fighter_index, &mut report);
            }
        }

        let effect_context = EffectTickContext {
            flash_active: self.state.screen_flash != 0,
            hit_stop_active: self.state.clock_stopped,
            camera_x: self.state.camera_x,
            camera_y: self.config.effect_camera_y,
        };
        report.effects = self.state.tick_effects(effect_context);

        self.state.tick_index = self.state.tick_index.saturating_add(1);
        self.last_report = report;
    }
}

impl SimulationTick for FightSimulation {
    fn tick(&mut self, input: InputFrame) {
        self.run_tick([input, InputFrame::default()]);
    }
}

#[cfg(test)]
mod rot_runtime_integration_tests {
    use super::*;
    use crate::combat_runtime::{BodyBox, FighterState};

    fn fighter(character_id: u8, action_id: u16) -> FighterState {
        FighterState {
            sprite_frame: 0,
            character_id,
            action_id,
            action_frame: 0,
            action_duration: 30,
            facing: crate::combat_runtime::SIDE_RIGHT,
            force_strength: 0,
            defending: true,
            invulnerable: false,
            hit_count: 0,
            hit_timer: 0,
            attack_damage: 0,
            angular_velocity: 0,
            attack_active: false,
            walking_forward: false,
            attack_box: None,
            is_kick: false,
            recoil: 0,
            hitstun: 0,
            rotation_step: 0,
            vertical_velocity: 0,
            in_air: false,
            x: 100.0,
            y: 240.0,
            speed: 5.0,
            distance_ai: 100,
            health: 100.0,
            power: 0,
            charge_power: 0.0,
            jump_direction: 0,
            low_time: 0,
            back_time: 0,
            sequence_state: 0,
            sequence_timer: 0,
            punch_charge: 0,
            kick_charge: 0,
            body: BodyBox {
                x1: -18.0,
                y1: 30.0,
                x2: 13.0,
                y2: 170.0,
            },
        }
    }

    fn simulation() -> FightSimulation {
        let state = CombatState {
            tick_index: 0,
            game_state: 7,
            camera_x: 0.0,
            clock_stopped: false,
            fighters: [fighter(9, ACTION_IDLE), fighter(2, ACTION_IDLE)],
            effects: Default::default(),
            pending_sound: None,
            screen_flash: 0,
            screen_shake: 0,
        };
        FightSimulation::new(
            state,
            FightSimulationConfig {
                special: ChavesSpecialContext {
                    charge_gain: 1.0,
                    charge_max: 100.0,
                    ground_y: 240.0,
                },
                frame: VictoryAnimationContext {
                    sprite_offset: 0,
                    ground_y: 240.0,
                    camera_x: 0.0,
                    camera_left_limit: 0.0,
                    camera_right_limit: 450.0,
                },
                effect_camera_y: 0.0,
            },
            2026,
        )
    }

    #[test]
    fn chapolin_package_registers_selects_and_runs_a_native_fight_timeline() {
        use crate::custom_character::{CustomCharacterPackage, PackageFile};

        let mut fight = simulation();
        let package = CustomCharacterPackage::from_files([PackageFile {
            path: "character.json".to_owned(),
            bytes: include_bytes!("../examples/custom-characters/chapolin/character.json").to_vec(),
        }])
        .unwrap();

        assert_eq!(fight.register_custom_character_package(package).unwrap(), 16);
        assert_eq!(fight.select_custom_character(0, "chapolin").unwrap(), 16);
        fight.queue_custom_character_move(0, "special_2", 2).unwrap();
        fight.tick_with_inputs([InputFrame::default(), InputFrame::default()]);

        let chapolin = &fight.state.fighters[0];
        assert_eq!(chapolin.character_id, 16);
        assert_eq!(chapolin.action_id, ACTION_SPECIAL2);
        assert_eq!(chapolin.action_frame, 1);
        assert_eq!(chapolin.sprite_frame, 40);
        assert_eq!(chapolin.x, 106.0);
        assert_eq!(fight.last_report().custom_character_frame_executions, vec![(0, 16)]);
        assert!(fight.last_report().errors.is_empty());
    }

    #[test]
    fn chapolin_special_one_emits_its_declared_effect_on_frame_ten() {
        use crate::custom_character::{CustomCharacterPackage, PackageFile};

        let mut fight = simulation();
        let package = CustomCharacterPackage::from_files([PackageFile {
            path: "character.json".to_owned(),
            bytes: include_bytes!("../examples/custom-characters/chapolin/character.json").to_vec(),
        }])
        .unwrap();
        fight.register_custom_character_package(package).unwrap();
        fight.select_custom_character(0, "chapolin").unwrap();
        fight.queue_custom_character_move(0, "special_1", 1).unwrap();

        for _ in 0..11 {
            fight.tick_with_inputs([InputFrame::default(), InputFrame::default()]);
        }

        assert!(fight.last_report().custom_character_events.iter().any(|event| {
            matches!(
                event,
                CustomCharacterEvent::SpawnEffect {
                    fighter_index: 0,
                    effect_id: 37,
                    variant: Some('A'),
                    ..
                }
            )
        }));
    }

    #[test]
    fn chapolin_final_move_obeys_power_gate_and_invulnerability_timeline() {
        use crate::custom_character::{CustomCharacterPackage, PackageFile};

        let mut fight = simulation();
        let package = CustomCharacterPackage::from_files([PackageFile {
            path: "character.json".to_owned(),
            bytes: include_bytes!("../examples/custom-characters/chapolin/character.json").to_vec(),
        }])
        .unwrap();
        fight.register_custom_character_package(package).unwrap();
        fight.select_custom_character(0, "chapolin").unwrap();

        assert!(fight.queue_custom_character_move(0, "special_final", 1).is_err());
        fight.state.fighters[0].power = 1;
        fight.queue_custom_character_move(0, "special_final", 1).unwrap();
        fight.tick_with_inputs([InputFrame::default(), InputFrame::default()]);
        assert!(fight.state.fighters[0].invulnerable);
        assert_eq!(fight.state.fighters[0].power, 0);
    }

    #[test]
    fn rot_host_recursively_executes_nested_acao_jogador_for_player_two() {
        let mut fight = simulation();
        let common = RotDocument::from_str(
            "Comum.rot",
            include_str!("../research/pascal-rot/Comum.rot"),
        )
        .unwrap();
        let caller =
            RotDocument::from_str("fixture.rot", "[Quadro]\nAcaoJogador(2, cmSocoFraco);").unwrap();
        let program = caller.parse_optional("Quadro").unwrap().unwrap();
        let mut host = RotCombatHost {
            state: &mut fight.state,
            rng: &mut fight.rng,
            player_index: 0,
            charge_gain: 1.0,
            charge_max: 100.0,
            ground_y: 240.0,
            common: &common,
            recursion_depth: 0,
        };
        let mut frame = RotExecutionFrame::new(&program, 1, 0).unwrap();
        frame.execute_with_host(&program, &mut host).unwrap();

        assert_eq!(host.state.fighters[1].action_id, ACTION_WEAK_PUNCH);
        assert_eq!(host.state.fighters[1].action_duration, 6);
        assert_eq!(host.state.pending_sound, Some(5));
        assert_eq!(host.recursion_depth, 0);
    }

    #[test]
    fn rot_host_reads_and_writes_golpe_ativo_on_both_live_fighters() {
        let mut fight = simulation();
        let common = RotDocument::from_str(
            "Comum.rot",
            include_str!("../research/pascal-rot/Comum.rot"),
        )
        .unwrap();
        fight.state.fighters[0].attack_active = true;
        fight.state.fighters[1].attack_active = false;

        let mut host = RotCombatHost {
            state: &mut fight.state,
            rng: &mut fight.rng,
            player_index: 0,
            charge_gain: 1.0,
            charge_max: 100.0,
            ground_y: 240.0,
            common: &common,
            recursion_depth: 0,
        };

        assert!(matches!(
            host.read_fighter_field('P', "GolpeAtivo"),
            Ok(RotValue::Boolean(true))
        ));
        assert!(matches!(
            host.read_fighter_field('O', "GolpeAtivo"),
            Ok(RotValue::Boolean(false))
        ));

        host.write_fighter_field('P', "GolpeAtivo", RotValue::Boolean(false))
            .unwrap();
        host.write_fighter_field('P', "TempoAcertos", rot_int(44))
            .unwrap();
        host.write_fighter_field('P', "DanoGolpe", rot_int(18))
            .unwrap();
        host.write_fighter_field('P', "VelAngular", rot_int(-25))
            .unwrap();
        host.write_fighter_field('O', "GolpeAtivo", RotValue::Boolean(true))
            .unwrap();

        assert!(matches!(
            host.read_fighter_field('P', "GolpeAtivo"),
            Ok(RotValue::Boolean(false))
        ));
        assert_eq!(
            host.read_fighter_field('P', "TempoAcertos").unwrap(),
            rot_int(44)
        );
        assert_eq!(
            host.read_fighter_field('P', "DanoGolpe").unwrap(),
            rot_int(18)
        );
        assert_eq!(
            host.read_fighter_field('P', "VelAngular").unwrap(),
            rot_int(-25)
        );
        assert!(matches!(
            host.read_fighter_field('O', "GolpeAtivo"),
            Ok(RotValue::Boolean(true))
        ));
    }

    #[test]
    fn c12_esp2_weak_and_strong_chain_from_special6_only_after_attack_deactivation() {
        for (command_code, button, expected_force) in [
            (18, AttackButton::WeakPunch, 1),
            (19, AttackButton::StrongPunch, 2),
        ] {
            let mut fight = simulation();
            fight.state.fighters[0].character_id = 12;
            fight.state.fighters[0].action_id = crate::combat_runtime::ACTION_SPECIAL6;
            fight.state.fighters[0].attack_active = false;
            let mut report = FightTickReport::default();

            fight.dispatch_character_command(0, button, command_code, &mut report);

            let fighter = &fight.state.fighters[0];
            assert_eq!(fighter.action_id, ACTION_SPECIAL2);
            assert_eq!(fighter.action_frame, 0);
            assert_eq!(fighter.action_duration, 20);
            assert_eq!(fighter.force_strength, expected_force);
            assert_eq!(fighter.charge_power, 6.0);
            assert!(!fighter.attack_active);
            assert_eq!(report.rot_action_executions, vec![(0, 12, command_code)]);
            assert!(report.unhandled_commands.is_empty());
            assert!(report.errors.is_empty());
        }
    }

    #[test]
    fn c12_esp2_does_not_chain_from_special6_while_attack_is_still_active() {
        for (command_code, button) in [
            (18, AttackButton::WeakPunch),
            (19, AttackButton::StrongPunch),
        ] {
            let mut fight = simulation();
            fight.state.fighters[0].character_id = 12;
            fight.state.fighters[0].action_id = crate::combat_runtime::ACTION_SPECIAL6;
            fight.state.fighters[0].action_frame = 7;
            fight.state.fighters[0].force_strength = 2;
            fight.state.fighters[0].charge_power = 9.0;
            fight.state.fighters[0].attack_active = true;
            // Common [Acao] always clears Defendendo before this character guard.
            fight.state.fighters[0].defending = false;
            let before = fight.state.fighters[0].clone();
            let mut report = FightTickReport::default();

            fight.dispatch_character_command(0, button, command_code, &mut report);

            assert_eq!(fight.state.fighters[0], before);
            assert_eq!(report.rot_action_executions, vec![(0, 12, command_code)]);
            assert!(report.unhandled_commands.is_empty());
            assert!(report.errors.is_empty());
        }
    }

    #[test]
    fn c12_esp2_keeps_the_original_idle_allowlist_even_when_attack_flag_is_set() {
        let mut fight = simulation();
        fight.state.fighters[0].character_id = 12;
        fight.state.fighters[0].action_id = ACTION_IDLE;
        fight.state.fighters[0].attack_active = true;
        let mut report = FightTickReport::default();

        fight.dispatch_character_command(0, AttackButton::WeakPunch, 18, &mut report);

        let fighter = &fight.state.fighters[0];
        assert_eq!(fighter.action_id, ACTION_SPECIAL2);
        assert_eq!(fighter.action_duration, 20);
        assert_eq!(fighter.force_strength, 1);
        assert_eq!(fighter.charge_power, 6.0);
        assert!(fighter.attack_active);
        assert_eq!(report.rot_action_executions, vec![(0, 12, 18)]);
        assert!(report.unhandled_commands.is_empty());
        assert!(report.errors.is_empty());
    }

    #[test]
    fn c09_special_three_executes_pinned_rot_with_live_state_rng_and_effect_helper() {
        let mut fight = simulation();
        let mut report = FightTickReport::default();

        // Pascal cmEsp3Fraco = 20. This goes through the normal character
        // command dispatcher, not a Rust implementation of the special.
        fight.dispatch_character_command(0, AttackButton::WeakPunch, 20, &mut report);

        let fighter = &fight.state.fighters[0];
        assert_eq!(fighter.action_id, ACTION_SPECIAL3);
        assert_eq!(fighter.action_frame, 0);
        assert_eq!(fighter.action_duration, 15);
        assert_eq!(fighter.force_strength, 1);
        assert_eq!(fighter.charge_power, 5.0);
        assert!(!fighter.defending);
        assert_eq!(fight.state.pending_sound, Some(45));
        assert_eq!(fight.state.effects.active_count(), 5);
        assert_eq!(fight.rng_calls_consumed(), 10);
        assert_eq!(report.rot_action_executions, vec![(0, 9, 20)]);
        assert!(report.unhandled_commands.is_empty());
        assert!(report.errors.is_empty());
    }

    #[test]
    fn common_action_prelude_and_epilogue_run_around_a_guarded_character_script() {
        let mut fight = simulation();
        fight.state.fighters[0].action_id = ACTION_WALKING;
        fight.state.fighters[0].action_frame = 6;
        fight.state.fighters[0].vertical_velocity = 1;
        fight.state.fighters[0].defending = true;

        // cmApanharAlto is rejected by the common grounded guard because
        // VelY != 0. The character script leaves the fighter walking; the
        // shared [AcaoFim] must still advance QuadroAcao.
        fight.execute_character_rot_action(0, 9, 8).unwrap();

        let fighter = &fight.state.fighters[0];
        assert_eq!(fighter.action_id, ACTION_WALKING);
        assert_eq!(fighter.action_frame, 7);
        assert!(!fighter.defending);
    }

    #[test]
    fn c04_final_special_executes_rot_and_writes_the_real_opponents_invulnerability() {
        let mut fight = simulation();
        fight.state.fighters[0].character_id = 4;
        fight.state.fighters[0].power = 1;
        fight.state.fighters[1].invulnerable = true;
        let mut report = FightTickReport::default();

        // Pascal cmEspFinal = 24. C04.rot writes O.Invulneravel := False.
        fight.dispatch_character_command(0, AttackButton::StrongKick, 24, &mut report);

        let player = &fight.state.fighters[0];
        let opponent = &fight.state.fighters[1];
        assert_eq!(player.action_id, ACTION_SPECIAL_FINAL);
        assert_eq!(player.action_frame, 0);
        assert_eq!(player.action_duration, 13);
        assert_eq!(player.force_strength, 2);
        assert!(player.is_kick);
        assert_eq!(player.power, 0);
        assert!(!opponent.invulnerable);
        assert_eq!(fight.state.screen_flash, 22);
        assert_eq!(fight.state.pending_sound, Some(19));
        assert_eq!(report.rot_action_executions, vec![(0, 4, 24)]);
        assert!(report.unhandled_commands.is_empty());
        assert!(report.errors.is_empty());
    }
    #[test]
    fn c01_special_three_executes_the_real_rot_action_section() {
        let mut fight = simulation();
        fight.state.fighters[0].character_id = 1;
        let mut report = FightTickReport::default();

        fight.dispatch_character_command(0, AttackButton::WeakPunch, 20, &mut report);

        let fighter = &fight.state.fighters[0];
        assert_eq!(fighter.action_id, ACTION_SPECIAL3);
        assert_eq!(fighter.action_frame, 0);
        assert_eq!(fighter.action_duration, 10);
        assert_eq!(fighter.force_strength, 1);
        assert_eq!(fighter.charge_power, 6.0);
        assert_eq!(fighter.y, 240.0);
        assert!(!fighter.in_air);
        assert!(!fighter.defending);
        assert_eq!(report.rot_action_executions, vec![(0, 1, 20)]);
        assert!(report.special_outcomes.is_empty());
        assert!(report.unhandled_commands.is_empty());
        assert!(report.errors.is_empty());
    }

    #[test]
    fn c01_final_special_executes_real_rot_and_preserves_live_host_writes() {
        let mut fight = simulation();
        fight.state.fighters[0].character_id = 1;
        fight.state.fighters[0].power = 1;
        let mut report = FightTickReport::default();

        fight.dispatch_character_command(0, AttackButton::StrongKick, 24, &mut report);

        let fighter = &fight.state.fighters[0];
        assert_eq!(fighter.action_id, ACTION_SPECIAL_FINAL);
        assert_eq!(fighter.action_frame, 0);
        assert_eq!(fighter.action_duration, 38);
        assert_eq!(fighter.force_strength, 3);
        assert_eq!(fighter.power, 0);
        assert_eq!(fighter.sprite_frame, 36);
        assert_eq!(fight.state.screen_flash, 22);
        assert_eq!(fight.state.pending_sound, Some(19));
        assert_eq!(report.rot_action_executions, vec![(0, 1, 24)]);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn c13_jaiminho_routes_standard_action_entries_through_rot() {
        for (command_code, button, action, duration) in [
            (14, AttackButton::WeakPunch, ACTION_VICTORY, 250),
            (16, AttackButton::WeakPunch, ACTION_SPECIAL1, 28),
            (17, AttackButton::StrongPunch, ACTION_SPECIAL1, 28),
            (18, AttackButton::WeakPunch, ACTION_SPECIAL2, 16),
            (19, AttackButton::StrongPunch, ACTION_SPECIAL2, 16),
            (20, AttackButton::WeakKick, ACTION_SPECIAL3, 30),
            (21, AttackButton::StrongKick, ACTION_SPECIAL3, 30),
            (22, AttackButton::WeakKick, ACTION_SPECIAL4, 30),
            (23, AttackButton::StrongKick, ACTION_SPECIAL4, 30),
            (24, AttackButton::StrongKick, ACTION_SPECIAL4, 30),
        ] {
            let mut fight = simulation();
            fight.state.fighters[0].character_id = 13;
            if command_code == 18 || command_code == 19 {
                fight.state.fighters[0].vertical_velocity = -5;
                fight.state.fighters[0].jump_direction = 2;
            }
            let mut report = FightTickReport::default();

            fight.dispatch_character_command(0, button, command_code, &mut report);

            let fighter = &fight.state.fighters[0];
            assert_eq!(fighter.action_id, action, "C13 command {command_code}");
            assert_eq!(fighter.action_frame, 0);
            assert_eq!(fighter.action_duration, duration);
            assert_eq!(report.rot_action_executions, vec![(0, 13, command_code)]);
            assert!(report.unhandled_commands.is_empty());
            assert!(report.errors.is_empty());
            match command_code {
                16 | 17 | 20 | 21 | 22 | 23 | 24 => {
                    assert_eq!(fighter.charge_power, 5.0);
                }
                _ => {}
            }
            if command_code == 18 || command_code == 19 {
                assert_eq!(fighter.vertical_velocity, -10);
                assert_eq!(fighter.jump_direction, 0);
                assert_eq!(
                    fighter.force_strength,
                    if command_code == 18 { 1 } else { 2 }
                );
            }
            if command_code == 16 || command_code == 17 {
                assert_eq!(
                    fighter.force_strength,
                    if command_code == 16 { 1 } else { 2 }
                );
            }
            if command_code == 20 || command_code == 21 {
                assert_eq!(
                    fighter.force_strength,
                    if command_code == 20 { 1 } else { 2 }
                );
            }
            if command_code == 22 || command_code == 23 || command_code == 24 {
                assert_eq!(
                    fighter.force_strength,
                    match command_code {
                        22 => 1,
                        23 => 2,
                        _ => 3,
                    }
                );
            }
        }
    }

    #[test]
    fn c14_dona_neves_routes_supported_action_entries_through_rot() {
        for (command_code, button, action, duration) in [
            (14, AttackButton::WeakPunch, ACTION_VICTORY, 250),
            (16, AttackButton::WeakPunch, ACTION_SPECIAL1, 28),
            (17, AttackButton::StrongPunch, ACTION_SPECIAL1, 28),
            (18, AttackButton::WeakPunch, ACTION_SPECIAL2, 35),
            (19, AttackButton::StrongPunch, ACTION_SPECIAL2, 35),
            (20, AttackButton::WeakKick, ACTION_SPECIAL3, 35),
            (21, AttackButton::StrongKick, ACTION_SPECIAL3, 35),
            (22, AttackButton::WeakKick, ACTION_SPECIAL4, 35),
            (23, AttackButton::StrongKick, ACTION_SPECIAL4, 35),
            (24, AttackButton::StrongKick, ACTION_SPECIAL_FINAL, 70),
        ] {
            let mut fight = simulation();
            fight.state.fighters[0].character_id = 14;
            if command_code == 20 || command_code == 21 {
                fight.state.fighters[0].action_id = ACTION_JUMPING;
                fight.state.fighters[0].in_air = true;
                fight.state.fighters[0].vertical_velocity = -5;
            }
            if command_code == 24 {
                fight.state.fighters[0].power = 1;
            }
            let mut report = FightTickReport::default();

            fight.dispatch_character_command(0, button, command_code, &mut report);

            let fighter = &fight.state.fighters[0];
            assert_eq!(fighter.action_id, action, "C14 command {command_code}");
            assert_eq!(fighter.action_frame, 0);
            assert_eq!(fighter.action_duration, duration);
            assert_eq!(report.rot_action_executions, vec![(0, 14, command_code)]);
            assert!(report.unhandled_commands.is_empty());
            assert!(report.errors.is_empty());
            if (16..=23).contains(&command_code) {
                assert_eq!(fighter.charge_power, 5.0);
            }
            if command_code == 20 || command_code == 21 || command_code == 22 || command_code == 23
            {
                assert!(!fighter.in_air);
                assert_eq!(fighter.vertical_velocity, 0);
            }
            if command_code == 24 {
                assert_eq!(fighter.power, 0);
                assert_eq!(fighter.sprite_frame, 37);
                assert_eq!(fight.state.screen_flash, 22);
                assert_eq!(fight.state.pending_sound, Some(19));
            }
        }
    }

    #[test]
    fn c15_gloria_routes_supported_action_entries_through_rot() {
        for (command_code, button, action, duration) in [
            (14, AttackButton::WeakPunch, ACTION_VICTORY, 250),
            (16, AttackButton::WeakPunch, ACTION_SPECIAL1, 28),
            (17, AttackButton::StrongPunch, ACTION_SPECIAL1, 28),
            (18, AttackButton::WeakPunch, ACTION_SPECIAL2, 22),
            (19, AttackButton::StrongPunch, ACTION_SPECIAL2, 22),
            (20, AttackButton::WeakKick, ACTION_SPECIAL3, 25),
            (21, AttackButton::StrongKick, ACTION_SPECIAL3, 25),
            (22, AttackButton::WeakKick, ACTION_SPECIAL4, 18),
            (23, AttackButton::StrongKick, ACTION_SPECIAL4, 18),
            (24, AttackButton::StrongKick, ACTION_SPECIAL_FINAL, 30),
        ] {
            let mut fight = simulation();
            fight.state.fighters[0].character_id = 15;
            if command_code == 24 {
                fight.state.fighters[0].power = 1;
            }
            let mut report = FightTickReport::default();

            fight.dispatch_character_command(0, button, command_code, &mut report);

            let fighter = &fight.state.fighters[0];
            assert_eq!(fighter.action_id, action, "C15 command {command_code}");
            assert_eq!(fighter.action_frame, 0);
            assert_eq!(fighter.action_duration, duration);
            assert_eq!(report.rot_action_executions, vec![(0, 15, command_code)]);
            assert!(report.unhandled_commands.is_empty());
            assert!(report.errors.is_empty());
            if (16..=23).contains(&command_code) {
                assert_eq!(fighter.charge_power, 5.0);
            }
            if command_code == 16 || command_code == 17 {
                assert_eq!(
                    fighter.force_strength,
                    if command_code == 16 { 1 } else { 2 }
                );
            }
            if command_code == 18 || command_code == 19 {
                assert_eq!(
                    fighter.force_strength,
                    if command_code == 18 { 1 } else { 2 }
                );
            }
            if command_code == 20 || command_code == 21 {
                assert_eq!(
                    fighter.force_strength,
                    if command_code == 20 { 1 } else { 2 }
                );
                assert!(!fighter.in_air);
            }
            if command_code == 22 || command_code == 23 {
                assert_eq!(
                    fighter.force_strength,
                    if command_code == 22 { 1 } else { 2 }
                );
            }
            if command_code == 24 {
                assert_eq!(fighter.power, 0);
                assert_eq!(fighter.sprite_frame, 12);
                assert_eq!(fight.state.screen_flash, 22);
                assert_eq!(fight.state.pending_sound, Some(19));
            }
        }
    }

    #[test]
    fn c13_extra_one_executes_real_rot_instead_of_the_compatibility_port() {
        let mut fight = simulation();
        fight.state.fighters[0].character_id = 13;
        let mut report = FightTickReport::default();

        fight.dispatch_character_command(0, AttackButton::WeakPunch, 25, &mut report);

        let fighter = &fight.state.fighters[0];
        assert_eq!(fighter.action_id, ACTION_SPECIAL4);
        assert_eq!(fighter.action_frame, 0);
        assert_eq!(fighter.action_duration, 30);
        assert_eq!(fighter.force_strength, 4);
        assert_eq!(fighter.charge_power, 5.0);
        assert_eq!(report.rot_action_executions, vec![(0, 13, 25)]);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn c13_extra_two_executes_real_rot_and_updates_live_opponent_and_globals() {
        let mut fight = simulation();
        fight.state.fighters[0].character_id = 13;
        fight.state.fighters[0].power = 1;
        fight.state.fighters[0].in_air = true;
        fight.state.fighters[0].vertical_velocity = -5;
        fight.state.fighters[0].jump_direction = 2;
        let mut report = FightTickReport::default();

        fight.dispatch_character_command(0, AttackButton::StrongKick, 26, &mut report);

        let fighter = &fight.state.fighters[0];
        assert_eq!(fighter.action_id, ACTION_SPECIAL_FINAL);
        assert_eq!(fighter.action_frame, 0);
        assert_eq!(fighter.action_duration, 30);
        assert_eq!(fighter.power, 0);
        assert_eq!(fighter.sprite_frame, 37);
        assert!(!fighter.in_air);
        assert_eq!(fighter.vertical_velocity, 0);
        assert_eq!(fighter.jump_direction, 0);
        assert_eq!(fight.state.screen_flash, 22);
        assert_eq!(fight.state.pending_sound, Some(19));
        assert_eq!(report.rot_action_executions, vec![(0, 13, 26)]);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn c01_rot_entry_matches_the_retained_c01_hand_port_for_special_three() {
        let mut rot = simulation();
        let mut hand_port = simulation();
        rot.state.fighters[0].character_id = 1;
        hand_port.state.fighters[0].character_id = 1;
        let mut rot_report = FightTickReport::default();

        rot.dispatch_character_command(0, AttackButton::WeakPunch, 20, &mut rot_report);
        hand_port
            .state
            .dispatch_chaves_special_code(0, 20, hand_port.config.special)
            .unwrap();

        assert_eq!(rot.state.fighters[0], hand_port.state.fighters[0]);
        assert_eq!(rot.state.pending_sound, hand_port.state.pending_sound);
        assert_eq!(rot.state.screen_flash, hand_port.state.screen_flash);
        assert_eq!(
            rot.state.effects.active_count(),
            hand_port.state.effects.active_count()
        );
        assert!(rot_report.errors.is_empty());
    }

    #[test]
    fn c01_rot_entry_matches_the_retained_c01_hand_port_for_final_special() {
        let mut rot = simulation();
        let mut hand_port = simulation();
        rot.state.fighters[0].character_id = 1;
        hand_port.state.fighters[0].character_id = 1;
        rot.state.fighters[0].power = 1;
        hand_port.state.fighters[0].power = 1;
        let mut rot_report = FightTickReport::default();

        rot.dispatch_character_command(0, AttackButton::StrongKick, 24, &mut rot_report);
        hand_port
            .state
            .dispatch_chaves_special_code(0, 24, hand_port.config.special)
            .unwrap();

        assert_eq!(rot.state.fighters[0], hand_port.state.fighters[0]);
        assert_eq!(rot.state.pending_sound, hand_port.state.pending_sound);
        assert_eq!(rot.state.screen_flash, hand_port.state.screen_flash);
        assert!(rot_report.errors.is_empty());
    }

    #[test]
    fn c13_rot_extras_match_the_retained_hand_port_oracle() {
        for command_code in [25, 26] {
            let mut rot = simulation();
            let mut compatibility = simulation();
            rot.state.fighters[0].character_id = 13;
            compatibility.state.fighters[0].character_id = 13;
            if command_code == 26 {
                rot.state.fighters[0].power = 1;
                compatibility.state.fighters[0].power = 1;
                rot.state.fighters[0].in_air = true;
                compatibility.state.fighters[0].in_air = true;
                rot.state.fighters[0].vertical_velocity = -5;
                compatibility.state.fighters[0].vertical_velocity = -5;
                rot.state.fighters[0].jump_direction = 2;
                compatibility.state.fighters[0].jump_direction = 2;
            }
            let mut rot_report = FightTickReport::default();
            let mut compatibility_report = FightTickReport::default();

            rot.dispatch_character_command(
                0,
                AttackButton::WeakPunch,
                command_code,
                &mut rot_report,
            );
            compatibility.state.fighters[0].defending = false;
            compatibility.dispatch_c13_extra_compatibility_oracle(
                command_code,
                0,
                &mut compatibility_report,
            );
            let _ = compatibility.state.finish_common_action(0);

            assert_eq!(
                rot.state.fighters[0], compatibility.state.fighters[0],
                "C13 command {command_code}"
            );
            assert_eq!(rot.state.screen_flash, compatibility.state.screen_flash);
            assert_eq!(rot.state.pending_sound, compatibility.state.pending_sound);
            assert!(rot_report.errors.is_empty());
        }
    }
    #[test]
    fn c08_bruja_routes_victory_pose_and_special_entries() {
        for (command_code, button, action, duration) in [
            (14, AttackButton::WeakPunch, ACTION_VICTORY, 250),
            (15, AttackButton::WeakPunch, ACTION_POSE, 20),
            (16, AttackButton::WeakPunch, ACTION_SPECIAL1, 28),
            (17, AttackButton::StrongPunch, ACTION_SPECIAL1, 28),
            (20, AttackButton::WeakKick, ACTION_SPECIAL3, 20),
            (21, AttackButton::StrongKick, ACTION_SPECIAL3, 20),
            (22, AttackButton::WeakKick, ACTION_SPECIAL4, 25),
            (23, AttackButton::StrongKick, ACTION_SPECIAL4, 25),
        ] {
            let mut fight = simulation();
            fight.state.fighters[0].character_id = 8;
            let mut report = FightTickReport::default();

            fight.dispatch_character_command(0, button, command_code, &mut report);

            let fighter = &fight.state.fighters[0];
            assert_eq!(fighter.action_id, action, "C08 command {command_code}");
            assert_eq!(fighter.action_frame, 0);
            assert_eq!(fighter.action_duration, duration);
            assert_eq!(report.rot_action_executions, vec![(0, 8, command_code)]);
            assert!(report.errors.is_empty());
        }

        for command_code in [18, 19] {
            let mut fight = simulation();
            fight.state.fighters[0].character_id = 8;
            fight.state.fighters[0].action_id = ACTION_JUMPING;
            let mut report = FightTickReport::default();
            let button = if command_code == 18 {
                AttackButton::WeakPunch
            } else {
                AttackButton::StrongPunch
            };

            fight.dispatch_character_command(0, button, command_code, &mut report);

            assert_eq!(fight.state.fighters[0].action_id, ACTION_SPECIAL2);
            assert_eq!(fight.state.fighters[0].action_duration, 15);
            assert_eq!(report.rot_action_executions, vec![(0, 8, command_code)]);
            assert!(report.errors.is_empty());
        }
    }

    #[test]
    fn c08_active_effect_guards_reject_first_and_second_specials() {
        for (command_code, effect_type, action) in [(16, 18, ACTION_IDLE), (18, 19, ACTION_JUMPING)]
        {
            let mut fight = simulation();
            fight.state.fighters[0].character_id = 8;
            fight.state.fighters[0].action_id = action;
            fight
                .state
                .effects
                .spawn(
                    effect_type,
                    Some('A'),
                    1,
                    EFFECT_SIDE_RIGHT,
                    100.0,
                    100.0,
                    &mut fight.rng,
                )
                .unwrap();
            let mut report = FightTickReport::default();

            fight.dispatch_character_command(0, AttackButton::WeakPunch, command_code, &mut report);

            assert_eq!(fight.state.fighters[0].action_id, action);
            assert_eq!(fight.state.fighters[0].charge_power, 0.0);
            assert_eq!(fight.state.effects.active_count(), 1);
            assert!(report.errors.is_empty());
        }
    }

    #[test]
    fn c08_final_special_consumes_power_and_writes_sprite_flash_and_sound() {
        let mut fight = simulation();
        fight.state.fighters[0].character_id = 8;
        fight.state.fighters[0].power = 1;
        let mut report = FightTickReport::default();

        fight.dispatch_character_command(0, AttackButton::StrongKick, 24, &mut report);

        let fighter = &fight.state.fighters[0];
        assert_eq!(fighter.action_id, ACTION_SPECIAL_FINAL);
        assert_eq!(fighter.action_frame, 0);
        assert_eq!(fighter.action_duration, 20);
        assert_eq!(fighter.power, 0);
        assert_eq!(fighter.sprite_frame, 33);
        assert_eq!(fight.state.screen_flash, 22);
        assert_eq!(fight.state.pending_sound, Some(19));
        assert_eq!(report.rot_action_executions, vec![(0, 8, 24)]);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn c11_godines_routes_victory_pose_and_all_special_entries() {
        for (command_code, button, action, duration) in [
            (14, AttackButton::WeakPunch, ACTION_VICTORY, 250),
            (15, AttackButton::WeakPunch, ACTION_POSE, 20),
            (16, AttackButton::WeakPunch, ACTION_SPECIAL1, 28),
            (17, AttackButton::StrongPunch, ACTION_SPECIAL1, 28),
            (18, AttackButton::WeakPunch, ACTION_SPECIAL2, 30),
            (19, AttackButton::StrongPunch, ACTION_SPECIAL2, 30),
            (20, AttackButton::WeakKick, ACTION_SPECIAL3, 9),
            (21, AttackButton::StrongKick, ACTION_SPECIAL3, 15),
            (22, AttackButton::WeakKick, ACTION_SPECIAL4, 12),
            (23, AttackButton::StrongKick, ACTION_SPECIAL4, 12),
            (24, AttackButton::WeakPunch, ACTION_SPECIAL_FINAL, 45),
        ] {
            let mut fight = simulation();
            fight.state.fighters[0].character_id = 11;
            if command_code == 24 {
                fight.state.fighters[0].power = 1;
            }
            let mut report = FightTickReport::default();
            fight.dispatch_character_command(0, button, command_code, &mut report);
            let fighter = &fight.state.fighters[0];
            assert_eq!(fighter.action_id, action, "C11 command {command_code}");
            assert_eq!(fighter.action_frame, 0);
            assert_eq!(fighter.action_duration, duration);
            assert_eq!(report.rot_action_executions, vec![(0, 11, command_code)]);
            assert!(report.unhandled_commands.is_empty());
            assert!(report.errors.is_empty());
            match command_code {
                16 | 17 => {
                    assert_eq!(fighter.charge_power, 5.0);
                    assert_eq!(
                        fighter.force_strength,
                        if command_code == 16 { 1 } else { 2 }
                    );
                }
                18 | 19 => {
                    assert_eq!(fighter.charge_power, 6.0);
                    assert_eq!(
                        fighter.force_strength,
                        if command_code == 18 { 1 } else { 2 }
                    );
                }
                20 | 21 => {
                    assert_eq!(fighter.charge_power, 5.0);
                    assert_eq!(
                        fighter.force_strength,
                        if command_code == 20 { 1 } else { 2 }
                    );
                }
                22 | 23 => {
                    assert_eq!(fighter.charge_power, 5.0);
                    assert_eq!(
                        fighter.force_strength,
                        if command_code == 22 { 1 } else { 2 }
                    );
                }
                24 => {
                    assert_eq!(fighter.power, 0);
                    assert_eq!(fighter.sprite_frame, 6);
                    assert_eq!(fight.state.screen_flash, 22);
                    assert_eq!(fight.state.pending_sound, Some(19));
                }
                _ => {}
            }
        }
    }

    #[test]
    fn c12_srfurtado_routes_supported_action_entries_and_preserves_final_special_writes() {
        for (command_code, button, action, duration) in [
            (14, AttackButton::WeakPunch, ACTION_VICTORY, 250),
            (15, AttackButton::WeakPunch, ACTION_POSE, 20),
            (16, AttackButton::WeakPunch, ACTION_SPECIAL1, 28),
            (17, AttackButton::StrongPunch, ACTION_SPECIAL1, 28),
            (20, AttackButton::WeakKick, ACTION_SPECIAL3, 28),
            (21, AttackButton::StrongKick, ACTION_SPECIAL3, 28),
            (22, AttackButton::WeakKick, ACTION_SPECIAL4, 32),
        ] {
            let mut fight = simulation();
            fight.state.fighters[0].character_id = 12;
            if command_code == 22 {
                fight.state.fighters[0].power = 1;
            }
            let mut report = FightTickReport::default();
            fight.dispatch_character_command(0, button, command_code, &mut report);
            let fighter = &fight.state.fighters[0];
            assert_eq!(fighter.action_id, action, "C12 command {command_code}");
            assert_eq!(fighter.action_frame, 0);
            assert_eq!(fighter.action_duration, duration);
            assert_eq!(report.rot_action_executions, vec![(0, 12, command_code)]);
            assert!(report.unhandled_commands.is_empty());
            assert!(report.errors.is_empty());
            if command_code == 22 {
                assert_eq!(fighter.power, 0);
                assert_eq!(fighter.sprite_frame, 38);
                assert_eq!(fight.state.screen_flash, 22);
                assert_eq!(fight.state.pending_sound, Some(19));
            }
        }
    }

    #[test]
    fn c10_nhonho_routes_victory_pose_and_all_special_entries() {
        for (command_code, button, action, duration) in [
            (14, AttackButton::WeakPunch, ACTION_VICTORY, 250),
            (15, AttackButton::WeakPunch, ACTION_POSE, 20),
            (16, AttackButton::WeakPunch, ACTION_SPECIAL1, 20),
            (17, AttackButton::StrongPunch, ACTION_SPECIAL1, 20),
            (18, AttackButton::WeakPunch, ACTION_SPECIAL2, 11),
            (19, AttackButton::StrongPunch, ACTION_SPECIAL2, 20),
            (20, AttackButton::WeakKick, ACTION_SPECIAL3, 20),
            (21, AttackButton::StrongKick, ACTION_SPECIAL3, 20),
            (22, AttackButton::WeakKick, ACTION_SPECIAL4, 13),
            (23, AttackButton::StrongKick, ACTION_SPECIAL4, 13),
        ] {
            let mut fight = simulation();
            fight.state.fighters[0].character_id = 10;
            fight.state.fighters[0].punch_charge = 9;
            let mut report = FightTickReport::default();

            fight.dispatch_character_command(0, button, command_code, &mut report);

            let fighter = &fight.state.fighters[0];
            assert_eq!(fighter.action_id, action, "C10 command {command_code}");
            assert_eq!(fighter.action_frame, 0);
            assert_eq!(fighter.action_duration, duration);
            assert_eq!(report.rot_action_executions, vec![(0, 10, command_code)]);
            assert!(report.unhandled_commands.is_empty());
            assert!(report.errors.is_empty());

            match command_code {
                16 | 17 => {
                    assert_eq!(fighter.punch_charge, 0);
                    assert_eq!(fighter.charge_power, 5.0);
                    assert_eq!(
                        fighter.force_strength,
                        if command_code == 16 { 1 } else { 2 }
                    );
                    assert_eq!(fight.state.pending_sound, Some(47));
                }
                18 | 19 => {
                    assert_eq!(fighter.charge_power, 6.0);
                    assert_eq!(
                        fighter.force_strength,
                        if command_code == 18 { 1 } else { 2 }
                    );
                    assert_eq!(fight.state.pending_sound, Some(48));
                }
                20 | 21 => {
                    assert_eq!(fighter.y, 190.0);
                    assert!(fighter.in_air);
                    assert_eq!(
                        fighter.vertical_velocity,
                        if command_code == 20 { -25 } else { -28 }
                    );
                    assert_eq!(fighter.charge_power, 5.0);
                    assert_eq!(
                        fighter.force_strength,
                        if command_code == 20 { 1 } else { 2 }
                    );
                    assert_eq!(fight.state.pending_sound, Some(49));
                }
                22 | 23 => {
                    assert_eq!(fighter.charge_power, 5.0);
                    assert_eq!(
                        fighter.force_strength,
                        if command_code == 22 { 1 } else { 2 }
                    );
                    assert_eq!(fight.state.pending_sound, Some(49));
                }
                _ => {}
            }
        }
    }

    #[test]
    fn c10_nhonho_final_special_consumes_power_and_writes_sprite_flash_and_sound() {
        let mut fight = simulation();
        fight.state.fighters[0].character_id = 10;
        fight.state.fighters[0].power = 3;
        fight.state.fighters[0].charge_power = 25.0;
        let mut report = FightTickReport::default();

        fight.dispatch_character_command(0, AttackButton::StrongKick, 24, &mut report);

        let fighter = &fight.state.fighters[0];
        assert_eq!(fighter.action_id, ACTION_SPECIAL_FINAL);
        assert_eq!(fighter.action_frame, 0);
        assert_eq!(fighter.action_duration, 40);
        assert_eq!(fighter.charge_power, 0.0);
        assert_eq!(fighter.power, 2);
        assert_eq!(fighter.sprite_frame, 38);
        assert_eq!(fight.state.screen_flash, 22);
        assert_eq!(fight.state.pending_sound, Some(19));
        assert_eq!(report.rot_action_executions, vec![(0, 10, 24)]);
        assert!(report.unhandled_commands.is_empty());
        assert!(report.errors.is_empty());
    }

    #[test]
    fn c10_nhonho_rejects_special_three_above_the_source_height_threshold() {
        let mut fight = simulation();
        fight.state.fighters[0].character_id = 10;
        fight.state.fighters[0].y = 149.0;
        let mut report = FightTickReport::default();

        fight.dispatch_character_command(0, AttackButton::WeakKick, 20, &mut report);

        let fighter = &fight.state.fighters[0];
        assert_eq!(fighter.action_id, ACTION_IDLE);
        assert_eq!(fighter.y, 149.0);
        assert!(!fighter.in_air);
        assert_eq!(fighter.charge_power, 0.0);
        assert_eq!(fight.state.pending_sound, None);
        assert_eq!(report.rot_action_executions, vec![(0, 10, 20)]);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn c10_nhonho_rejects_final_special_without_low_health_or_power() {
        let mut fight = simulation();
        fight.state.fighters[0].character_id = 10;
        fight.state.fighters[0].power = 0;
        fight.state.fighters[0].health = 100.0;
        let mut report = FightTickReport::default();

        fight.dispatch_character_command(0, AttackButton::StrongKick, 24, &mut report);

        let fighter = &fight.state.fighters[0];
        assert_eq!(fighter.action_id, ACTION_IDLE);
        assert_eq!(fighter.power, 0);
        assert_eq!(fighter.sprite_frame, 0);
        assert_eq!(fight.state.screen_flash, 0);
        assert_eq!(fight.state.pending_sound, None);
        assert_eq!(report.rot_action_executions, vec![(0, 10, 24)]);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn c07_sr_barriga_routes_victory_pose_and_all_special_entries() {
        for (command_code, button, action, duration) in [
            (14, AttackButton::WeakPunch, ACTION_VICTORY, 250),
            (15, AttackButton::WeakPunch, ACTION_POSE, 20),
            (16, AttackButton::WeakPunch, ACTION_SPECIAL1, 17),
            (17, AttackButton::StrongPunch, ACTION_SPECIAL1, 20),
            (18, AttackButton::WeakKick, ACTION_SPECIAL2, 15),
            (19, AttackButton::StrongKick, ACTION_SPECIAL2, 15),
            (20, AttackButton::WeakKick, ACTION_SPECIAL3, 15),
            (21, AttackButton::StrongKick, ACTION_SPECIAL3, 15),
            (22, AttackButton::WeakKick, ACTION_SPECIAL4, 15),
            (23, AttackButton::StrongKick, ACTION_SPECIAL4, 15),
        ] {
            let mut fight = simulation();
            fight.state.fighters[0].character_id = 7;
            fight.state.fighters[0].punch_charge = 8;
            let mut report = FightTickReport::default();

            fight.dispatch_character_command(0, button, command_code, &mut report);

            let fighter = &fight.state.fighters[0];
            assert_eq!(fighter.action_id, action, "C07 command {command_code}");
            assert_eq!(fighter.action_frame, 0);
            assert_eq!(fighter.action_duration, duration);
            assert_eq!(report.rot_action_executions, vec![(0, 7, command_code)]);
            assert!(report.errors.is_empty());
            if matches!(command_code, 16 | 18) {
                assert_eq!(fighter.punch_charge, 0);
            }
            if matches!(command_code, 20..=23) {
                assert!(fighter.in_air);
                assert_eq!(fighter.jump_direction, 0);
                assert_eq!(fighter.low_time, 0);
                assert_eq!(fighter.y, 240.0);
                let expected_velocity = match command_code {
                    20 | 22 => -27,
                    21 => -32,
                    23 => -30,
                    _ => unreachable!(),
                };
                assert_eq!(fighter.vertical_velocity, expected_velocity);
            }
            if matches!(command_code, 16 | 17 | 18 | 19) {
                assert_eq!(fighter.punch_charge, 0);
            }
        }
    }

    #[test]
    fn c07_final_special_consumes_power_and_uses_source_sprite_and_effect_writes() {
        let mut fight = simulation();
        fight.state.fighters[0].character_id = 7;
        fight.state.fighters[0].power = 1;
        let mut report = FightTickReport::default();

        fight.dispatch_character_command(0, AttackButton::StrongKick, 24, &mut report);

        let fighter = &fight.state.fighters[0];
        assert_eq!(fighter.action_id, ACTION_SPECIAL_FINAL);
        assert_eq!(fighter.action_frame, 0);
        assert_eq!(fighter.action_duration, 12);
        assert_eq!(fighter.power, 0);
        assert_eq!(fighter.sprite_frame, 38);
        assert_eq!(fight.state.screen_flash, 22);
        assert_eq!(fight.state.pending_sound, Some(19));
        assert_eq!(report.rot_action_executions, vec![(0, 7, 24)]);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn c06_professor_girafales_routes_victory_pose_and_weak_strong_specials() {
        for (command_code, button, action, duration) in [
            (14, AttackButton::WeakPunch, ACTION_VICTORY, 250),
            (15, AttackButton::WeakPunch, ACTION_POSE, 20),
            (16, AttackButton::WeakPunch, ACTION_SPECIAL1, 28),
            (17, AttackButton::StrongPunch, ACTION_SPECIAL1, 28),
            (18, AttackButton::WeakKick, ACTION_SPECIAL2, 28),
            (19, AttackButton::StrongKick, ACTION_SPECIAL2, 28),
            (20, AttackButton::WeakPunch, ACTION_SPECIAL3, 25),
            (21, AttackButton::StrongPunch, ACTION_SPECIAL3, 25),
            (22, AttackButton::WeakKick, ACTION_SPECIAL4, 30),
            (23, AttackButton::StrongKick, ACTION_SPECIAL4, 30),
        ] {
            let mut fight = simulation();
            fight.state.fighters[0].character_id = 6;
            let mut report = FightTickReport::default();

            fight.dispatch_character_command(0, button, command_code, &mut report);

            let fighter = &fight.state.fighters[0];
            assert_eq!(fighter.action_id, action, "C06 command {command_code}");
            assert_eq!(fighter.action_frame, 0);
            assert_eq!(fighter.action_duration, duration);
            assert_eq!(report.rot_action_executions, vec![(0, 6, command_code)]);
            assert!(report.errors.is_empty());
        }
    }

    #[test]
    fn c06_final_special_executes_rot_and_applies_airborne_power_writes() {
        let mut fight = simulation();
        fight.state.fighters[0].character_id = 6;
        fight.state.fighters[0].power = 1;
        let mut report = FightTickReport::default();

        fight.dispatch_character_command(0, AttackButton::StrongKick, 24, &mut report);

        let fighter = &fight.state.fighters[0];
        assert_eq!(fighter.action_id, ACTION_SPECIAL_FINAL);
        assert_eq!(fighter.action_frame, 0);
        assert_eq!(fighter.action_duration, 50);
        assert_eq!(fighter.vertical_velocity, -18);
        assert!(fighter.in_air);
        assert_eq!(fighter.power, 0);
        assert_eq!(fighter.sprite_frame, 41);
        assert_eq!(fight.state.screen_flash, 22);
        assert_eq!(fight.state.pending_sound, Some(19));
        assert_eq!(report.rot_action_executions, vec![(0, 6, 24)]);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn c05_active_effect_13_rejects_esp1_without_writing_action_state() {
        let mut fight = simulation();
        fight.state.fighters[0].character_id = 5;
        fight
            .state
            .effects
            .spawn(
                13,
                Some('A'),
                1,
                EFFECT_SIDE_RIGHT,
                100.0,
                100.0,
                &mut fight.rng,
            )
            .unwrap();
        let mut report = FightTickReport::default();

        fight.dispatch_character_command(0, AttackButton::WeakPunch, 16, &mut report);

        let fighter = &fight.state.fighters[0];
        assert_eq!(fighter.action_id, ACTION_IDLE);
        assert_eq!(fighter.charge_power, 0.0);
        assert_eq!(fight.state.pending_sound, None);
        assert_eq!(fight.state.effects.active_count(), 1);
        assert_eq!(report.rot_action_executions, vec![(0, 5, 16)]);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn c06_active_effect_15_rejects_both_first_and_second_specials() {
        for command_code in [16, 17, 18, 19] {
            let mut fight = simulation();
            fight.state.fighters[0].character_id = 6;
            fight
                .state
                .effects
                .spawn(
                    15,
                    Some('A'),
                    1,
                    EFFECT_SIDE_RIGHT,
                    100.0,
                    100.0,
                    &mut fight.rng,
                )
                .unwrap();
            let mut report = FightTickReport::default();

            fight.dispatch_character_command(0, AttackButton::WeakPunch, command_code, &mut report);

            assert_eq!(fight.state.fighters[0].action_id, ACTION_IDLE);
            assert_eq!(fight.state.fighters[0].charge_power, 0.0);
            assert_eq!(fight.state.effects.active_count(), 1);
            assert!(report.errors.is_empty());
        }
    }

    #[test]
    fn c05_victory_and_pose_entries_execute_the_real_rot_action_section() {
        for (command_code, button, expected_action, duration) in [
            (14, AttackButton::WeakPunch, ACTION_VICTORY, 250),
            (15, AttackButton::WeakPunch, ACTION_POSE, 20),
        ] {
            let mut fight = simulation();
            fight.state.fighters[0].character_id = 5;
            let mut report = FightTickReport::default();

            fight.dispatch_character_command(0, button, command_code, &mut report);

            let fighter = &fight.state.fighters[0];
            assert_eq!(fighter.action_id, expected_action);
            assert_eq!(fighter.action_frame, 0);
            assert_eq!(fighter.action_duration, duration);
            assert_eq!(report.rot_action_executions, vec![(0, 5, command_code)]);
            assert!(report.errors.is_empty());
        }
    }

    #[test]
    fn c05_special_one_weak_and_strong_execute_rot() {
        for (command_code, button, force) in [
            (16, AttackButton::WeakPunch, 1),
            (17, AttackButton::StrongPunch, 2),
        ] {
            let mut fight = simulation();
            fight.state.fighters[0].character_id = 5;
            let mut report = FightTickReport::default();

            fight.dispatch_character_command(0, button, command_code, &mut report);

            let fighter = &fight.state.fighters[0];
            assert_eq!(fighter.action_id, ACTION_SPECIAL1);
            assert_eq!(fighter.action_frame, 0);
            assert_eq!(fighter.action_duration, 28);
            assert_eq!(fighter.force_strength, force);
            assert_eq!(fighter.charge_power, 5.0);
            assert_eq!(fight.state.pending_sound, Some(29));
            assert_eq!(report.rot_action_executions, vec![(0, 5, command_code)]);
            assert!(report.errors.is_empty());
        }
    }

    #[test]
    fn c05_special_two_weak_and_strong_execute_rot() {
        for (command_code, button, force) in [
            (18, AttackButton::WeakPunch, 1),
            (19, AttackButton::StrongPunch, 2),
        ] {
            let mut fight = simulation();
            fight.state.fighters[0].character_id = 5;
            let mut report = FightTickReport::default();

            fight.dispatch_character_command(0, button, command_code, &mut report);

            let fighter = &fight.state.fighters[0];
            assert_eq!(fighter.action_id, ACTION_SPECIAL2);
            assert_eq!(fighter.action_duration, 23);
            assert_eq!(fighter.force_strength, force);
            assert_eq!(fighter.charge_power, 5.0);
            assert_eq!(fight.state.pending_sound, Some(30));
            assert_eq!(report.rot_action_executions, vec![(0, 5, command_code)]);
            assert!(report.errors.is_empty());
        }
    }

    #[test]
    fn c05_special_three_preserves_landing_guard_and_all_grounding_writes() {
        for (command_code, button, force, duration) in [
            (20, AttackButton::WeakKick, 1, 38),
            (21, AttackButton::StrongKick, 2, 40),
        ] {
            let mut fight = simulation();
            fight.state.fighters[0].character_id = 5;
            fight.state.fighters[0].action_id = ACTION_JUMPING;
            fight.state.fighters[0].y = 200.0;
            fight.state.fighters[0].vertical_velocity = -8;
            fight.state.fighters[0].in_air = true;
            fight.state.fighters[0].low_time = 7;
            let mut report = FightTickReport::default();

            fight.dispatch_character_command(0, button, command_code, &mut report);

            let fighter = &fight.state.fighters[0];
            assert_eq!(fighter.action_id, ACTION_SPECIAL3);
            assert_eq!(fighter.y, 240.0);
            assert_eq!(fighter.vertical_velocity, 0);
            assert!(!fighter.in_air);
            assert_eq!(fighter.low_time, 0);
            assert_eq!(fighter.action_duration, duration);
            assert_eq!(fighter.force_strength, force);
            assert_eq!(fighter.charge_power, 6.0);
            assert_eq!(fight.state.pending_sound, Some(31));
            assert_eq!(report.rot_action_executions, vec![(0, 5, command_code)]);
            assert!(report.errors.is_empty());
        }
    }

    #[test]
    fn c05_special_three_rejects_when_above_the_landing_height_limit() {
        let mut fight = simulation();
        fight.state.fighters[0].character_id = 5;
        fight.state.fighters[0].action_id = ACTION_JUMPING;
        fight.state.fighters[0].y = 100.0;
        fight.state.fighters[0].vertical_velocity = -8;
        fight.state.fighters[0].in_air = true;
        let mut report = FightTickReport::default();

        fight.dispatch_character_command(0, AttackButton::WeakKick, 20, &mut report);

        let fighter = &fight.state.fighters[0];
        assert_eq!(fighter.action_id, ACTION_JUMPING);
        assert_eq!(fighter.y, 100.0);
        assert_eq!(fighter.vertical_velocity, -8);
        assert!(fighter.in_air);
        assert_eq!(fighter.charge_power, 0.0);
        assert_eq!(report.rot_action_executions, vec![(0, 5, 20)]);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn c05_special_four_resets_kick_charge_and_preserves_weak_strong_force() {
        for (command_code, button, force) in [
            (22, AttackButton::WeakKick, 1),
            (23, AttackButton::StrongKick, 2),
        ] {
            let mut fight = simulation();
            fight.state.fighters[0].character_id = 5;
            fight.state.fighters[0].kick_charge = 9;
            let mut report = FightTickReport::default();

            fight.dispatch_character_command(0, button, command_code, &mut report);

            let fighter = &fight.state.fighters[0];
            assert_eq!(fighter.action_id, ACTION_SPECIAL4);
            assert_eq!(fighter.action_frame, 0);
            assert_eq!(fighter.action_duration, 20);
            assert_eq!(fighter.kick_charge, 0);
            assert_eq!(fighter.force_strength, force);
            assert_eq!(fighter.charge_power, 5.0);
            assert_eq!(fight.state.pending_sound, Some(32));
            assert_eq!(report.rot_action_executions, vec![(0, 5, command_code)]);
            assert!(report.errors.is_empty());
        }
    }

    #[test]
    fn c05_final_special_executes_rot_and_writes_power_sprite_flash_and_sound() {
        let mut fight = simulation();
        fight.state.fighters[0].character_id = 5;
        fight.state.fighters[0].power = 1;
        let mut report = FightTickReport::default();

        fight.dispatch_character_command(0, AttackButton::StrongKick, 24, &mut report);

        let fighter = &fight.state.fighters[0];
        assert_eq!(fighter.action_id, ACTION_SPECIAL_FINAL);
        assert_eq!(fighter.action_frame, 0);
        assert_eq!(fighter.action_duration, 30);
        assert_eq!(fighter.power, 0);
        assert_eq!(fighter.sprite_frame, 6);
        assert_eq!(fight.state.screen_flash, 22);
        assert_eq!(fight.state.pending_sound, Some(19));
        assert_eq!(report.rot_action_executions, vec![(0, 5, 24)]);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn c02_special_one_weak_executes_the_real_rot_action_section() {
        let mut fight = simulation();
        fight.state.fighters[0].character_id = 2;
        let mut report = FightTickReport::default();

        fight.dispatch_character_command(0, AttackButton::WeakPunch, 16, &mut report);

        let fighter = &fight.state.fighters[0];
        assert_eq!(fighter.action_id, ACTION_SPECIAL1);
        assert_eq!(fighter.action_frame, 0);
        assert_eq!(fighter.action_duration, 28);
        assert_eq!(fighter.force_strength, 1);
        assert_eq!(fighter.charge_power, 5.0);
        assert!(!fighter.defending);
        assert_eq!(report.rot_action_executions, vec![(0, 2, 16)]);
        assert!(report.unhandled_commands.is_empty());
        assert!(report.errors.is_empty());
    }

    #[test]
    fn c02_special_one_strong_uses_the_same_rot_branch_with_strong_force() {
        let mut fight = simulation();
        fight.state.fighters[0].character_id = 2;
        let mut report = FightTickReport::default();

        fight.dispatch_character_command(0, AttackButton::StrongPunch, 17, &mut report);

        let fighter = &fight.state.fighters[0];
        assert_eq!(fighter.action_id, ACTION_SPECIAL1);
        assert_eq!(fighter.action_frame, 0);
        assert_eq!(fighter.action_duration, 28);
        assert_eq!(fighter.force_strength, 2);
        assert_eq!(fighter.charge_power, 5.0);
        assert_eq!(report.rot_action_executions, vec![(0, 2, 17)]);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn c02_special_one_keeps_its_source_guard_and_still_runs_common_prelude() {
        let mut fight = simulation();
        fight.state.fighters[0].character_id = 2;
        fight.state.fighters[0].action_id = ACTION_SPECIAL2;
        fight.state.fighters[0].defending = true;
        let mut report = FightTickReport::default();

        fight.dispatch_character_command(0, AttackButton::WeakPunch, 16, &mut report);

        let fighter = &fight.state.fighters[0];
        assert_eq!(fighter.action_id, ACTION_SPECIAL2);
        assert_eq!(fighter.charge_power, 0.0);
        assert!(!fighter.defending);
        assert_eq!(report.rot_action_executions, vec![(0, 2, 16)]);
        assert!(report.errors.is_empty());
    }
    #[test]
    fn c02_special_two_weak_and_strong_execute_their_rot_branch() {
        for (command_code, button, force) in [
            (18, AttackButton::WeakPunch, 1),
            (19, AttackButton::StrongPunch, 2),
        ] {
            let mut fight = simulation();
            fight.state.fighters[0].character_id = 2;
            let mut report = FightTickReport::default();

            fight.dispatch_character_command(0, button, command_code, &mut report);

            let fighter = &fight.state.fighters[0];
            assert_eq!(fighter.action_id, ACTION_SPECIAL2, "command {command_code}");
            assert_eq!(fighter.action_frame, 0);
            assert_eq!(fighter.action_duration, 30);
            assert_eq!(fighter.force_strength, force);
            assert_eq!(fighter.charge_power, 6.0);
            assert_eq!(report.rot_action_executions, vec![(0, 2, command_code)]);
            assert!(report.errors.is_empty());
        }
    }

    #[test]
    fn c02_special_three_weak_and_strong_execute_rot_and_clear_air_state() {
        for (command_code, button, force) in [
            (20, AttackButton::WeakKick, 1),
            (21, AttackButton::StrongKick, 2),
        ] {
            let mut fight = simulation();
            fight.state.fighters[0].character_id = 2;
            fight.state.fighters[0].action_id = ACTION_JUMPING;
            fight.state.fighters[0].in_air = true;
            let mut report = FightTickReport::default();

            fight.dispatch_character_command(0, button, command_code, &mut report);

            let fighter = &fight.state.fighters[0];
            assert_eq!(fighter.action_id, ACTION_SPECIAL3, "command {command_code}");
            assert_eq!(fighter.action_frame, 0);
            assert_eq!(fighter.action_duration, 20);
            assert_eq!(fighter.force_strength, force);
            assert_eq!(fighter.charge_power, 5.0);
            assert!(fighter.is_kick);
            assert!(!fighter.in_air);
            assert_eq!(fight.state.pending_sound, Some(21));
            assert_eq!(report.rot_action_executions, vec![(0, 2, command_code)]);
            assert!(report.errors.is_empty());
        }
    }

    #[test]
    fn c02_special_four_weak_executes_rot_and_applies_charge_gain() {
        let mut fight = simulation();
        fight.state.fighters[0].character_id = 2;
        let mut report = FightTickReport::default();

        fight.dispatch_character_command(0, AttackButton::WeakKick, 22, &mut report);

        let fighter = &fight.state.fighters[0];
        assert_eq!(fighter.action_id, ACTION_SPECIAL4);
        assert_eq!(fighter.action_frame, 0);
        assert_eq!(fighter.action_duration, 45);
        assert_eq!(fighter.charge_power, 7.0);
        assert_eq!(report.rot_action_executions, vec![(0, 2, 22)]);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn c02_final_special_executes_rot_and_writes_power_flash_and_sound() {
        let mut fight = simulation();
        fight.state.fighters[0].character_id = 2;
        fight.state.fighters[0].power = 1;
        let mut report = FightTickReport::default();

        fight.dispatch_character_command(0, AttackButton::StrongKick, 23, &mut report);

        let fighter = &fight.state.fighters[0];
        assert_eq!(fighter.action_id, 30); // Pascal acEspecial6
        assert_eq!(fighter.action_frame, 0);
        assert_eq!(fighter.action_duration, 13);
        assert_eq!(fighter.force_strength, 2);
        assert_eq!(fighter.power, 0);
        assert_eq!(fighter.sprite_frame, 44);
        assert_eq!(fight.state.screen_flash, 22);
        assert_eq!(fight.state.pending_sound, Some(19));
        assert_eq!(report.rot_action_executions, vec![(0, 2, 23)]);
        assert!(report.errors.is_empty());
    }
    #[test]
    fn c03_special_one_weak_and_strong_execute_the_rot_section() {
        for (command_code, button, force) in [
            (16, AttackButton::WeakPunch, 1),
            (17, AttackButton::StrongPunch, 2),
        ] {
            let mut fight = simulation();
            fight.state.fighters[0].character_id = 3;
            let mut report = FightTickReport::default();

            fight.dispatch_character_command(0, button, command_code, &mut report);

            let fighter = &fight.state.fighters[0];
            assert_eq!(fighter.action_id, ACTION_SPECIAL1);
            assert_eq!(fighter.action_duration, 28);
            assert_eq!(fighter.force_strength, force);
            assert_eq!(fighter.charge_power, 5.0);
            assert_eq!(fight.state.pending_sound, Some(23));
            assert_eq!(report.rot_action_executions, vec![(0, 3, command_code)]);
            assert!(report.errors.is_empty());
        }
    }

    #[test]
    fn c03_special_two_weak_and_strong_execute_rot_air_state_writes() {
        for (command_code, button, force, velocity) in [
            (18, AttackButton::WeakPunch, 1, -23),
            (19, AttackButton::StrongPunch, 2, -29),
        ] {
            let mut fight = simulation();
            fight.state.fighters[0].character_id = 3;
            let mut report = FightTickReport::default();

            fight.dispatch_character_command(0, button, command_code, &mut report);

            let fighter = &fight.state.fighters[0];
            assert_eq!(fighter.action_id, ACTION_SPECIAL2);
            assert_eq!(fighter.action_duration, 15);
            assert_eq!(fighter.force_strength, force);
            assert_eq!(fighter.vertical_velocity, velocity);
            assert!(fighter.is_kick);
            assert!(fighter.in_air);
            assert_eq!(fighter.charge_power, 5.0);
            assert_eq!(fight.state.pending_sound, Some(24));
            assert_eq!(report.rot_action_executions, vec![(0, 3, command_code)]);
            assert!(report.errors.is_empty());
        }
    }

    #[test]
    fn c03_special_three_weak_and_strong_use_source_specific_durations() {
        for (command_code, button, force, duration) in [
            (20, AttackButton::WeakKick, 1, 14),
            (21, AttackButton::StrongKick, 2, 19),
        ] {
            let mut fight = simulation();
            fight.state.fighters[0].character_id = 3;
            let mut report = FightTickReport::default();

            fight.dispatch_character_command(0, button, command_code, &mut report);

            let fighter = &fight.state.fighters[0];
            assert_eq!(fighter.action_id, ACTION_SPECIAL3);
            assert_eq!(fighter.action_duration, duration);
            assert_eq!(fighter.force_strength, force);
            assert!(fighter.is_kick);
            assert_eq!(fighter.charge_power, 5.0);
            assert_eq!(fight.state.pending_sound, Some(24));
            assert_eq!(report.rot_action_executions, vec![(0, 3, command_code)]);
            assert!(report.errors.is_empty());
        }
    }

    #[test]
    fn c03_special_four_weak_and_strong_execute_rot_launch_writes() {
        for (command_code, button, velocity) in [
            (22, AttackButton::WeakKick, -22),
            (23, AttackButton::StrongKick, -23),
        ] {
            let mut fight = simulation();
            fight.state.fighters[0].character_id = 3;
            let mut report = FightTickReport::default();

            fight.dispatch_character_command(0, button, command_code, &mut report);

            let fighter = &fight.state.fighters[0];
            assert_eq!(fighter.action_id, ACTION_SPECIAL4);
            assert_eq!(fighter.action_duration, 15);
            assert_eq!(fighter.vertical_velocity, velocity);
            assert!(fighter.in_air);
            assert_eq!(fighter.charge_power, 6.0);
            assert_eq!(report.rot_action_executions, vec![(0, 3, command_code)]);
            assert!(report.errors.is_empty());
        }
    }

    #[test]
    fn c03_final_special_executes_rot_and_writes_power_flash_and_sound() {
        let mut fight = simulation();
        fight.state.fighters[0].character_id = 3;
        fight.state.fighters[0].power = 1;
        let mut report = FightTickReport::default();

        fight.dispatch_character_command(0, AttackButton::StrongKick, 24, &mut report);

        let fighter = &fight.state.fighters[0];
        assert_eq!(fighter.action_id, 29); // Pascal acEspecial5
        assert_eq!(fighter.sprite_frame, 48);
        assert_eq!(fighter.action_frame, 0);
        assert_eq!(fighter.action_duration, 15);
        assert_eq!(fighter.power, 0);
        assert_eq!(fight.state.screen_flash, 22);
        assert_eq!(fight.state.pending_sound, Some(19));
        assert_eq!(report.rot_action_executions, vec![(0, 3, 24)]);
        assert!(report.errors.is_empty());
    }
    #[test]
    fn all_fifteen_original_fighters_execute_their_live_quadro_scripts() {
        for character_id in 1..=15 {
            let mut fight = simulation();
            fight.state.fighters[0].character_id = character_id;
            fight.frame_boxes[0] = Some(load_fighter_frame_boxes(character_id).unwrap());
            fight.state.fighters[0].action_id = ACTION_IDLE;
            let mut report = FightTickReport::default();

            fight.advance_fighter_frame(0, &mut report);

            assert!(
                report.errors.is_empty(),
                "C{character_id:02} [Quadro] failed: {:?}",
                report.errors
            );
            assert_eq!(
                report.rot_frame_executions,
                vec![(0, character_id)],
                "C{character_id:02} did not execute its pinned [Quadro]"
            );
            assert_eq!(
                report.box_updates,
                vec![0],
                "C{character_id:02} boxes were not updated"
            );
        }
    }

    #[test]
    fn c08_special_five_clears_opponent_invulnerability_during_quadro() {
        let mut fight = simulation();
        fight.state.fighters[0].character_id = 8;
        fight.frame_boxes[0] = Some(load_fighter_frame_boxes(8).unwrap());
        fight.state.fighters[0].action_id = ACTION_SPECIAL_FINAL;
        // Odd frames avoid C08's effect-spawn branch; the invulnerability write is unconditional.
        fight.state.fighters[0].action_frame = 1;
        fight.state.fighters[1].invulnerable = true;
        let mut report = FightTickReport::default();

        fight.advance_fighter_frame(0, &mut report);

        assert!(report.errors.is_empty(), "{:?}", report.errors);
        assert!(!fight.state.fighters[1].invulnerable);
    }

    #[test]
    fn c13_special_five_clears_opponent_invulnerability_only_after_combo_hit_two() {
        for (hit_count, expected_invulnerable) in [(1, true), (2, false)] {
            let mut fight = simulation();
            fight.state.fighters[0].character_id = 13;
            fight.frame_boxes[0] = Some(load_fighter_frame_boxes(13).unwrap());
            fight.state.fighters[0].action_id = ACTION_SPECIAL_FINAL;
            fight.state.fighters[0].action_frame = 0;
            fight.state.fighters[0].hit_count = hit_count;
            fight.state.fighters[1].invulnerable = true;
            let mut report = FightTickReport::default();

            fight.advance_fighter_frame(0, &mut report);

            assert!(report.errors.is_empty(), "C13: {:?}", report.errors);
            assert_eq!(
                fight.state.fighters[1].invulnerable, expected_invulnerable,
                "C13 with P.Acertos={hit_count}"
            );
        }
    }

    #[test]
    fn c14_quadro_does_not_invent_opponent_side_effects() {
        let mut fight = simulation();
        fight.state.fighters[0].character_id = 14;
        fight.frame_boxes[0] = Some(load_fighter_frame_boxes(14).unwrap());
        fight.state.fighters[0].action_id = ACTION_SPECIAL_FINAL;
        fight.state.fighters[0].action_frame = 0;
        fight.state.fighters[1].invulnerable = true;
        let opponent_before = fight.state.fighters[1].clone();
        let mut report = FightTickReport::default();

        fight.advance_fighter_frame(0, &mut report);

        assert!(report.errors.is_empty(), "C14: {:?}", report.errors);
        assert_eq!(fight.state.fighters[1], opponent_before);
    }

    #[test]
    fn aplicar_caixas_selects_pascal_strength_rows_in_original_ini_order() {
        let mut fight = simulation();
        fight.state.fighters[0].character_id = 11;
        fight.frame_boxes[0] = Some(load_fighter_frame_boxes(11).unwrap());
        {
            let fighter = &mut fight.state.fighters[0];
            fighter.action_id = 27;
            fighter.action_frame = 1;
            fighter.force_strength = 1;
            fighter.facing = crate::combat_runtime::SIDE_RIGHT;
        }

        fight.apply_fighter_boxes(0).unwrap();
        let weak = fight.state.fighters[0].attack_box.unwrap();
        assert_eq!(
            weak.bounds,
            crate::RawBounds {
                x1: 10,
                y1: 130,
                x2: 82,
                y2: 158,
            }
        );
        assert!(weak.active);
        assert_eq!(weak.damage, 18);
        assert_eq!(fight.state.fighters[0].force_strength, 1);

        fight.state.fighters[0].force_strength = 2;
        fight.apply_fighter_boxes(0).unwrap();
        let strong = fight.state.fighters[0].attack_box.unwrap();
        assert_eq!(
            strong.bounds,
            crate::RawBounds {
                x1: 10,
                y1: 25,
                x2: 70,
                y2: 55,
            }
        );
        assert_eq!(strong.damage, 18);
        assert_eq!(fight.state.fighters[0].force_strength, 2);
    }

    #[test]
    fn aplicar_caixas_retains_attack_record_and_active_flag_on_lookup_miss() {
        let mut fight = simulation();
        fight.state.fighters[0].character_id = 11;
        fight.frame_boxes[0] = Some(load_fighter_frame_boxes(11).unwrap());
        {
            let fighter = &mut fight.state.fighters[0];
            fighter.action_id = 27;
            fighter.action_frame = 1;
            fighter.force_strength = 2;
            fighter.facing = crate::combat_runtime::SIDE_RIGHT;
        }
        fight.apply_fighter_boxes(0).unwrap();
        let previous = fight.state.fighters[0].attack_box;
        assert!(fight.state.fighters[0].attack_active);

        fight.state.fighters[0].action_frame = 99;
        fight.apply_fighter_boxes(0).unwrap();
        assert_eq!(fight.state.fighters[0].attack_box, previous);
        assert!(fight.state.fighters[0].attack_active);
    }

    #[test]
    fn aplicar_caixas_keeps_matched_inactive_rows_as_stored_boxes() {
        let mut fight = simulation();
        fight.state.fighters[0].character_id = 3;
        fight.frame_boxes[0] = Some(load_fighter_frame_boxes(3).unwrap());
        let fighter = &mut fight.state.fighters[0];
        fighter.action_id = 26;
        fighter.action_frame = 7;
        fighter.attack_active = true;

        fight.apply_fighter_boxes(0).unwrap();
        let stored = fight.state.fighters[0].attack_box.unwrap();
        assert_eq!(
            stored.bounds,
            crate::RawBounds {
                x1: 10,
                y1: 5,
                x2: 50,
                y2: 95,
            }
        );
        assert!(!stored.active);
        assert!(!fight.state.fighters[0].attack_active);
    }

    #[test]
    fn aplicar_caixas_mirrors_attack_bounds_without_mutating_the_source_table() {
        let mut fight = simulation();
        fight.state.fighters[0].character_id = 11;
        fight.frame_boxes[0] = Some(load_fighter_frame_boxes(11).unwrap());
        let fighter = &mut fight.state.fighters[0];
        fighter.action_id = 27;
        fighter.action_frame = 1;
        fighter.force_strength = 1;
        fighter.facing = SIDE_LEFT;

        fight.apply_fighter_boxes(0).unwrap();
        let selected = fight.state.fighters[0].attack_box.unwrap();
        assert_eq!(
            selected.bounds,
            crate::RawBounds {
                x1: -82,
                y1: 130,
                x2: -10,
                y2: 158,
            }
        );
        let source = fight.frame_boxes[0].as_ref().unwrap();
        assert_eq!(source.attacks[12].bounds.x1, 10);
    }
}
