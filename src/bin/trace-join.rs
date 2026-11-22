//! Reconcile a complete Pascal schema-v2 JSONL capture with the static
//! symbolic RNG/action-tick index. Writes JSONL: one summary followed by one
//! record per observed SortearValor request.

use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Write};
use std::process::ExitCode;

use serde_json::json;
use street_chaves::trace_join::{
    load_runtime_capture, load_symbolic_rng_sites, reconcile_capture, SiteMatchStatus,
};

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 || args.len() > 4 {
        return Err(
            "usage: trace-join <pascal-trace.jsonl> <symbolic-action-tick-index.json> [report.jsonl]"
                .into(),
        );
    }

    let capture_file = File::open(&args[1])?;
    let capture = load_runtime_capture(BufReader::new(capture_file))?;
    let symbolic_file = File::open(&args[2])?;
    let symbolic_sites = load_symbolic_rng_sites(BufReader::new(symbolic_file))?;
    let joined = reconcile_capture(&capture, &symbolic_sites);
    let unique_symbolic_sites: HashSet<_> = symbolic_sites
        .iter()
        .map(|site| site.rng_site.as_str())
        .collect();

    let mut unique = 0usize;
    let mut ambiguous = 0usize;
    let mut unmatched = 0usize;
    let mut non_rot_source = 0usize;
    let mut rot_expression_draws = 0usize;
    let mut pascal_helper_draws = 0usize;
    let mut pascal_direct_draws = 0usize;
    let source_column_draws = joined
        .iter()
        .filter(|row| row.draw.source_column.is_some())
        .count();
    let boolean_subexpression_records = capture
        .controls
        .iter()
        .filter(|control| control.control_type == "boolean_subexpression")
        .count();
    let short_circuit_skips = capture
        .controls
        .iter()
        .filter(|control| control.short_circuit == Some(true))
        .count();

    for row in &joined {
        match row.status {
            SiteMatchStatus::Unique => unique += 1,
            SiteMatchStatus::Ambiguous => ambiguous += 1,
            SiteMatchStatus::Unmatched => unmatched += 1,
            SiteMatchStatus::NonRotSource => non_rot_source += 1,
        }
        match row.draw.source_kind.as_str() {
            "rot_expression" => rot_expression_draws += 1,
            "pascal_helper" => pascal_helper_draws += 1,
            "pascal_direct" => pascal_direct_draws += 1,
            _ => {}
        }
    }

    let summary = json!({
        "record_type": "reconciliation_summary",
        "schema_version": 1,
        "session_id": capture.session_id,
        "runtime_schema_version": capture.schema_version,
        "runtime_draws": joined.len(),
        "symbolic_rng_timeline_occurrences": symbolic_sites.len(),
        "unique_symbolic_rng_sites": unique_symbolic_sites.len(),
        "unique": unique,
        "ambiguous": ambiguous,
        "unmatched": unmatched,
        "non_rot_source": non_rot_source,
        "source_kind_counts": {
            "rot_expression": rot_expression_draws,
            "pascal_helper": pascal_helper_draws,
            "pascal_direct": pascal_direct_draws
        },
        "match_policy": "fighter+character+section+source_line+width+literal_range_when_available;source_column_retained_for_follow_up",
        "source_column_draws": source_column_draws,
        "boolean_subexpression_records": boolean_subexpression_records,
        "short_circuit_skips": short_circuit_skips,
        "source_column_available_for_pascal_direct": true,
        "rot_expression_source_column_available": true,
        "instrumented_game_source_rng_scope": true,
        "external_rtl_library_rng_intercepted": false
    });

    let output: Box<dyn Write> = if args.len() == 4 {
        Box::new(BufWriter::new(File::create(&args[3])?))
    } else {
        Box::new(BufWriter::new(io::stdout()))
    };
    let mut output = output;

    writeln!(output, "{}", serde_json::to_string(&summary)?)?;

    for row in joined {
        let candidates: Vec<_> = row
            .candidates
            .iter()
            .map(|site| {
                json!({
                    "fighter_id": site.fighter_id,
                    "character": site.character,
                    "section": site.section,
                    "group_index": site.group_index,
                    "event_sequence": site.event_sequence,
                    "source_line": site.source_line,
                    "source_column": site.source_column,
                    "rng_site": site.rng_site,
                    "range_expression": site.range_expression,
                    "random_width": site.random_width,
                    "symbolic_action_guards": site.symbolic_action_guards,
                    "symbolic_tick_guards": site.symbolic_tick_guards,
                    "guards": site.guards,
                    "loop_context": site.loop_context
                })
            })
            .collect();

        let invocation_control_event_indices: Vec<_> = row
            .invocation_controls
            .iter()
            .map(|control| control.control_event_index)
            .collect();
        let record = json!({
            "record_type": "draw_reconciliation",
            "status": match row.status {
                SiteMatchStatus::Unique => "unique",
                SiteMatchStatus::Ambiguous => "ambiguous",
                SiteMatchStatus::Unmatched => "unmatched",
                SiteMatchStatus::NonRotSource => "non_rot_source",
            },
            "draw": {
                "api_call_index": row.draw.api_call_index,
                "tick_index": row.draw.tick_index,
                "script_execution_id": row.draw.script_execution_id,
                "parent_script_execution_id": row.draw.parent_script_execution_id,
                "script_draw_index": row.draw.script_draw_index,
                "fighter_index": row.draw.fighter_index,
                "character_id": row.draw.character_id,
                "action_id": row.draw.action_id,
                "action_frame": row.draw.action_frame,
                "command_code": row.draw.command_code,
                "source_kind": row.draw.source_kind,
                "source_origin": row.draw.source_origin,
                "source_line": row.draw.source_line,
                "source_column": row.draw.source_column,
                "width": row.draw.width,
                "range": row.draw.range,
                "result": row.draw.result,
                "first_mt_word_index": row.draw.first_mt_word_index,
                "raw_mt_words": row.draw.raw_mt_words
            },
            "candidate_count": candidates.len(),
            "candidates": candidates,
            "invocation_control_event_indices": invocation_control_event_indices
        });

        writeln!(output, "{}", serde_json::to_string(&record)?)?;
    }

    // Emit each boolean subexpression once instead of duplicating it per draw
    // in the same script invocation.
    for control in capture
        .controls
        .iter()
        .filter(|control| control.control_type == "boolean_subexpression")
    {
        let record = json!({
            "record_type": "boolean_subexpression",
            "session_id": capture.session_id,
            "control_event_index": control.control_event_index,
            "tick_index": control.tick_index,
            "script_execution_id": control.script_execution_id,
            "source_line": control.source_line,
            "source_column": control.source_column,
            "expression_node": control.expression_node,
            "operand_node": control.operand_node,
            "operand_source_line": control.operand_source_line,
            "operand_source_column": control.operand_source_column,
            "operator": control.operator,
            "operand": control.operand,
            "evaluated": control.evaluated,
            "boolean_value": control.boolean_value,
            "short_circuit": control.short_circuit,
            "expression_result": control.expression_result,
            "short_circuit_reason": control.short_circuit_reason
        });
        writeln!(output, "{}", serde_json::to_string(&record)?)?;
    }

    output.flush()?;
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("trace-join: {error}");
            ExitCode::FAILURE
        }
    }
}
