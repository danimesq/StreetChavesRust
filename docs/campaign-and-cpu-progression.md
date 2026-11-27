# Campaign and CPU progression

## Source contract

The campaign model follows the Pascal reference:

- `source/game/Estado4_Selecao.inc`: `criaadversario` generates the order.
- `source/game/Estado5_Adversarios.inc`: `CHS_ADVERSARIOS` presents opponents and chooses the next stage.
- `source/game/Estado7_Golpes.inc`: CPU probabilities depend on `Fase`.

For each selected fighter, the source generates seven opponents with `Random(FQtdLutadores) + 1`. It excludes the selected fighter, the fighter's `NaoEnfrenta` list and the configured boss. Every accepted opponent is removed from the available set, so the normal result has no repeats. Entry eight is the character-specific `Chefe`.

The source has a 2,001-attempt fallback per opponent. `CampaignPlan::generate` preserves that final-draw fallback. The data host must supply the roster size plus the selected fighter's `Chefe` and `NaoEnfrenta` metadata from its INI data.

## Runtime model

- `CampaignPlan` stores seven randomized opponents followed by the boss.
- `CampaignProgress` begins at phase 1, advances only after a victory and completes after winning phase 8. A defeat terminates the campaign.
- `FightSimulation::generate_campaign_plan` uses the fight's shared FPC 3.2.2 RNG stream for the seven opponent draws. `FightSimulation::choose_campaign_stage` uses that same stream for the independent alternative/rare rolls.
- `choose_campaign_stage` applies the primary stage, then the independent alternative and rare rolls in source order. A nonzero secret-stage override is applied last only in phases 1–7, as in Pascal; phase 8 keeps the boss's configured primary stage.
- `AdversariesPresentation` models the original `TempoEstado` clock: the state becomes ready for preparation after tick 150, and a skip edge adds five ticks.
- `FightSimulation::load_campaign_opponent` selects the current fighter for P2 and refreshes its command rules, source frame boxes and character ROT AI document. It also updates `cpu_campaign_phase`, so both original difficulty formulas change as the campaign advances.

The source formulas remain separate:

`CpuNovo = (8 - NivelJogo) * 0.6 + 1.3 - Fase * 0.1`

`defendergolpe = max(0, (8 - NivelJogo) * 0.9 + 1.2 - Fase * 0.1)`

## Current host boundary

This is the deterministic campaign model and CPU-reload seam, not a claim that the entire between-fight experience is finished. The presentation host still needs to:

1. Detect win/loss and call `CampaignProgress::record_victory` or `record_defeat`.
2. Enter and render `CHS_ADVERSARIOS` and drive its presentation clock.
3. Carry player state, apply the selected stage and enter `CHS_PREPARALUTA`.
4. Supply the fighter's `Chefe`, `NaoEnfrenta`, primary/alternative/rare stage metadata.

This preserves source-derived simulation rules without pretending that the existing fight loop already has a complete match-end state machine or a fully rendered adversaries screen.
