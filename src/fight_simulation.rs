//! First concrete, renderer-free fight orchestrator.
use crate::chaves_specials::{ChavesSpecialCommand, ChavesSpecialContext, ChavesSpecialOutcome};
use crate::combat_runtime::{
    C01FrameResult, CombatState, VictoryAnimationContext, ACTION_AIR_STRONG_KICK,
    ACTION_AIR_STRONG_PUNCH, ACTION_AIR_WEAK_KICK, ACTION_AIR_WEAK_PUNCH, ACTION_CROUCHING,
    ACTION_CROUCH_STRONG_KICK, ACTION_CROUCH_STRONG_PUNCH, ACTION_CROUCH_WEAK_KICK,
    ACTION_CROUCH_WEAK_PUNCH, ACTION_DEFENDING, ACTION_IDLE, ACTION_JUMPING, ACTION_LOW_DEFENDING,
    ACTION_POSE, ACTION_SPECIAL1, ACTION_SPECIAL2, ACTION_SPECIAL3, ACTION_SPECIAL4,
    ACTION_SPECIAL_FINAL, ACTION_STRONG_KICK, ACTION_STRONG_PUNCH, ACTION_VICTORY, ACTION_WALKING,
    ACTION_WEAK_KICK, ACTION_WEAK_PUNCH,
};
use crate::command_recognition::{AttackButton, CommandRecognizer};
use crate::effect_runtime::{EffectTickContext, EffectTickResult, EFFECT_SIDE_RIGHT};
use crate::fighter_command_data::{embedded_character_command_rules, CharacterCommandRules};
use crate::input::{Direction, InputFrame};
use crate::pascal_replay::PascalRandomSource;
use crate::rot_runtime::{RotDocument, RotExecutionFrame, RotExecutionHost, RotValue};
use crate::runtime::SimulationTick;
use crate::Fpc322Rng;

/// Bridge from Pascal ROT field/helper names to the authoritative fight state.
struct RotCombatHost<'a> {
    state: &'a mut CombatState,
    rng: &'a mut Fpc322Rng,
    player_index: usize,
    charge_gain: f64,
    charge_max: f64,
    ground_y: f64,
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
    pub c01_frames: Vec<(usize, C01FrameResult)>,
    pub effects: EffectTickResult,
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
    last_report: FightTickReport,
}

impl FightSimulation {
    /// Build a deterministic headless fight and load real per-character command tables.
    pub fn new(state: CombatState, config: FightSimulationConfig, rng_seed: i32) -> Self {
        let mut command_rules: [Option<CharacterCommandRules>; 2] = [None, None];
        let mut command_data_errors = Vec::new();
        for fighter_index in 0..2 {
            let character_id = state.fighters[fighter_index].character_id;
            match embedded_character_command_rules(character_id) {
                Ok(rules) => command_rules[fighter_index] = Some(rules),
                Err(error) => command_data_errors.push(error.to_string()),
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
            last_report: FightTickReport::default(),
        }
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
            && matches!(command_code, 25 | 26)
        {
            match self.execute_character_rot_action(fighter_index, 13, command_code) {
                Ok(()) => report
                    .rot_action_executions
                    .push((fighter_index, 13, command_code)),
                Err(error) => report.errors.push(format!(
                    "C13 [Acao] fighter {fighter_index}, command {command_code}: {error}"
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
            && matches!(command_code, 14..=17 | 20..=22)
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
        let frame_before = self.state.fighters[fighter_index].action_frame;

        match self
            .state
            .update_c01_character_frame(fighter_index, context, &mut self.rng)
        {
            Ok(frame_result) => {
                report.c01_frames.push((fighter_index, frame_result));
                let fighter = &mut self.state.fighters[fighter_index];
                if fighter.action_frame == frame_before
                    && fighter.action_frame < fighter.action_duration
                {
                    fighter.action_frame = fighter.action_frame.saturating_add(1);
                }
            }
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
                self.advance_c01_frame(fighter_index, &mut report);
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
            is_kick: false,
            recoil: 0,
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
                    assert_eq!(fighter.force_strength, if command_code == 16 { 1 } else { 2 });
                }
                18 | 19 => {
                    assert_eq!(fighter.charge_power, 6.0);
                    assert_eq!(fighter.force_strength, if command_code == 18 { 1 } else { 2 });
                }
                20 | 21 => {
                    assert_eq!(fighter.charge_power, 5.0);
                    assert_eq!(fighter.force_strength, if command_code == 20 { 1 } else { 2 });
                }
                22 | 23 => {
                    assert_eq!(fighter.charge_power, 5.0);
                    assert_eq!(fighter.force_strength, if command_code == 22 { 1 } else { 2 });
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
}
