use crate::error::{AppError, AppResult};
use crate::model::RETEST_HORIZON_STATUS_SCHEMA_VERSION;
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

const TRACKED_HORIZONS: &[&str] = &["1h", "4h", "24h"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetestHorizonStatusBuildOptions {
    pub generated_at_ms: i64,
    pub plan_file: Option<String>,
    pub driver_summary_file: Option<String>,
    pub checkpoint_s3_write: bool,
}

pub fn read_retest_horizon_plan(path: &Path) -> AppResult<Value> {
    if !path.is_absolute() {
        return Err(AppError::config(
            "retest horizon plan file must be an absolute path",
        ));
    }
    let raw = fs::read_to_string(path)?;
    read_retest_horizon_plan_from_bytes(&path.display().to_string(), raw.as_bytes())
}

pub fn read_retest_horizon_plan_from_bytes(label: &str, bytes: &[u8]) -> AppResult<Value> {
    let text =
        std::str::from_utf8(bytes).map_err(|error| AppError::Json(format!("{label}: {error}")))?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(AppError::validation(format!("{label} must not be empty")));
    }
    Ok(serde_json::from_str(trimmed)?)
}

pub fn build_retest_horizon_status(
    plan: &Value,
    driver_summary: Option<&Value>,
    options: &RetestHorizonStatusBuildOptions,
) -> AppResult<Value> {
    validate_plan(plan)?;
    let rows = plan
        .get("horizon_rows")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let driver = driver_summary.unwrap_or(&Value::Null);
    let latest_universe = driver
        .pointer("/manifest/latest_universe")
        .cloned()
        .unwrap_or(Value::Null);
    let latest_l1_as_of_ms = plan.get("latest_l1_as_of_ms").and_then(Value::as_i64);
    let candidate_ids = unique_sorted_strings(
        rows.iter()
            .filter_map(|row| string_field(row, "candidate_id")),
    );
    let replayed_candidate_ids = unique_sorted_strings(rows.iter().filter_map(|row| {
        (i64_field(row, "replay_run_count").unwrap_or(0) > 0)
            .then(|| string_field(row, "candidate_id"))
            .flatten()
    }));
    let plan_research_replay_completed = !candidate_ids.is_empty()
        && candidate_ids
            .iter()
            .all(|candidate_id| replayed_candidate_ids.contains(candidate_id));
    let promotion_passed = bool_pointer(driver, "/stage_state/promotion_passed")
        .or_else(|| bool_pointer(driver, "/stage_state/promoted"))
        .unwrap_or(false);
    let shadow_created = bool_pointer(driver, "/stage_state/shadow_created").unwrap_or(false);
    let paper_created = bool_pointer(driver, "/stage_state/paper_created").unwrap_or(false);
    let stage_state = json!({
        "candidate_generated": bool_pointer(driver, "/stage_state/candidate_generated").unwrap_or(!rows.is_empty()),
        "research_replay_completed": bool_pointer(driver, "/stage_state/research_replay_completed").unwrap_or(plan_research_replay_completed),
        "promotion_passed": promotion_passed,
        "shadow_created": shadow_created,
        "paper_created": paper_created,
        "live_enabled": false
    });

    let action_counts = action_counts(&rows);
    let ready_for_replay_count = count_actions(
        &rows,
        &[
            "run_research_replay_for_horizon",
            "materialize_completed_native_replay_sample",
            "accumulate_completed_native_replay_samples",
        ],
    );
    let waiting_for_market_l1_count = count_action(&rows, "wait_for_market_l1_horizon");
    let market_l1_coverage_extension_count =
        count_action(&rows, "extend_market_l1_horizon_coverage");
    let sample_accumulation_count =
        count_action(&rows, "accumulate_completed_native_replay_samples");
    let promotion_ready_for_review_count = count_action(&rows, "promotion_gate_ready_for_review");
    let symbols = unique_sorted_strings(
        rows.iter()
            .filter_map(|row| string_field(row, "primary_symbol").or_else(|| first_symbol(row))),
    );
    let horizon_summary = json!({
        "candidate_count": candidate_ids.len(),
        "horizon_count": rows.len(),
        "symbols": symbols,
        "next_action_counts": action_counts,
        "ready_for_replay_count": ready_for_replay_count,
        "waiting_for_market_l1_count": waiting_for_market_l1_count,
        "market_l1_coverage_extension_count": market_l1_coverage_extension_count,
        "sample_accumulation_count": sample_accumulation_count,
        "promotion_ready_for_review_count": promotion_ready_for_review_count
    });

    let next_wait_due_ms = min_ms_for_action(&rows, "wait_for_market_l1_horizon", "horizon_due_ms");
    let last_wait_due_ms = max_ms_for_action(&rows, "wait_for_market_l1_horizon", "horizon_due_ms");
    let oldest_accumulation_due_ms = min_ms_for_action(
        &rows,
        "accumulate_completed_native_replay_samples",
        "horizon_due_ms",
    );
    let latest_accumulation_due_ms = max_ms_for_action(
        &rows,
        "accumulate_completed_native_replay_samples",
        "horizon_due_ms",
    );
    let wait_deficit_ms = match (latest_l1_as_of_ms, next_wait_due_ms) {
        (Some(latest), Some(next)) => Some((next - latest).max(0)),
        _ => None,
    };
    let materialization_schedule = json!({
        "latest_l1_as_of_ms": latest_l1_as_of_ms,
        "latest_l1_as_of_iso": latest_l1_as_of_ms.map(iso8601_ms),
        "next_wait_horizon_due_ms": next_wait_due_ms,
        "next_wait_horizon_due_iso": next_wait_due_ms.map(iso8601_ms),
        "last_wait_horizon_due_ms": last_wait_due_ms,
        "last_wait_horizon_due_iso": last_wait_due_ms.map(iso8601_ms),
        "next_wait_deficit_ms": wait_deficit_ms,
        "oldest_accumulation_due_ms": oldest_accumulation_due_ms,
        "oldest_accumulation_due_iso": oldest_accumulation_due_ms.map(iso8601_ms),
        "latest_accumulation_due_ms": latest_accumulation_due_ms,
        "latest_accumulation_due_iso": latest_accumulation_due_ms.map(iso8601_ms)
    });

    let next_decision_verdict = next_decision_verdict(
        promotion_passed,
        promotion_ready_for_review_count,
        market_l1_coverage_extension_count,
        ready_for_replay_count,
        waiting_for_market_l1_count,
        sample_accumulation_count,
    );
    let safe_next_actions = safe_next_actions(
        promotion_passed,
        promotion_ready_for_review_count,
        market_l1_coverage_extension_count,
        ready_for_replay_count,
        waiting_for_market_l1_count,
        sample_accumulation_count,
    );
    let blocked_actions = blocked_actions(promotion_passed, shadow_created);
    let by_symbol = by_symbol(&rows);
    let by_horizon = by_horizon(&rows);
    let candidate_horizon_matrix = candidate_horizon_matrix(&rows);
    let status = json!({
        "schema_version": RETEST_HORIZON_STATUS_SCHEMA_VERSION,
        "generated_at_ms": options.generated_at_ms,
        "generated_at": iso8601_ms(options.generated_at_ms),
        "retest_horizon_plan_file": options.plan_file,
        "driver_summary_file": options.driver_summary_file,
        "safety": {
            "s3_write": false,
            "ecs_task_started": false,
            "dispatcher_mode_changed": false,
            "local_summary_only": true,
            "checkpoint_s3_write": options.checkpoint_s3_write,
            "shadow_paper_live_enabled": false
        },
        "stage_state": stage_state,
        "batch_state": batch_state(driver),
        "horizon_summary": horizon_summary,
        "materialization_schedule": materialization_schedule,
        "by_symbol": by_symbol,
        "by_horizon": by_horizon,
        "candidate_horizon_matrix_summary": candidate_horizon_matrix_summary(&candidate_horizon_matrix),
        "candidate_horizon_matrix": candidate_horizon_matrix,
        "next_decision": {
            "verdict": next_decision_verdict,
            "safe_next_actions": safe_next_actions,
            "scheduler_hint": {
                "latest_l1_as_of_ms": latest_l1_as_of_ms,
                "latest_l1_as_of_iso": latest_l1_as_of_ms.map(iso8601_ms),
                "run_research_after_l1_as_of_ms": next_wait_due_ms,
                "run_research_after_l1_as_of_iso": next_wait_due_ms.map(iso8601_ms),
                "wait_deficit_ms": wait_deficit_ms,
                "run_now_replay_ready": ready_for_replay_count > 0,
                "promotion_ready_for_review": promotion_ready_for_review_count > 0
            },
            "blocked_actions": blocked_actions
        },
        "verdict": next_decision_verdict,
        "selected_symbols": symbols,
        "next_action_counts": action_counts,
        "major50_state": major50_state(&latest_universe, driver, &rows),
        "research_factory_progression": research_factory_progression(&latest_universe, &rows, promotion_passed, shadow_created, paper_created),
        "coverage_gaps": coverage_gaps(&latest_universe, driver, &rows, shadow_created),
        "research_factory_gap_summary": research_factory_gap_summary(&latest_universe, driver, &rows, promotion_passed, shadow_created, paper_created, &safe_next_actions)
    });
    Ok(status)
}

fn validate_plan(plan: &Value) -> AppResult<()> {
    let rows = plan
        .get("horizon_rows")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::validation("retest horizon plan missing horizon_rows"))?;
    for (index, row) in rows.iter().enumerate() {
        for field in ["candidate_id", "horizon", "next_action"] {
            if string_field(row, field).is_none() {
                return Err(AppError::validation(format!(
                    "retest horizon plan horizon_rows[{index}] missing {field}"
                )));
            }
        }
    }
    Ok(())
}

fn batch_state(driver: &Value) -> Value {
    json!({
        "run_id": driver.get("run_id").cloned().unwrap_or(Value::Null),
        "universe_mode": driver.pointer("/manifest/universe_mode").cloned().unwrap_or(Value::Null),
        "dispatch_mode": driver.pointer("/manifest/dispatch_mode").cloned().unwrap_or(Value::Null),
        "selected_candidate_count": driver.pointer("/manifest/selected_candidate_count").cloned().unwrap_or(Value::Null),
        "eligible_candidate_pool_count": driver.pointer("/manifest/eligible_candidate_pool_count").cloned().unwrap_or(Value::Null),
        "selected_candidate_limit_reached": driver.pointer("/manifest/selected_candidate_limit_reached").cloned().unwrap_or(Value::Null),
        "unselected_eligible_candidate_count": driver.pointer("/manifest/unselected_eligible_candidate_count").cloned().unwrap_or(Value::Null),
        "selected_current_approved_candidate_count": driver.pointer("/manifest/selected_current_approved_candidate_count").cloned().unwrap_or(Value::Null),
        "research_report_status": driver.pointer("/report/research_run_status").cloned().unwrap_or(Value::Null),
        "source_candidate_count": driver.pointer("/report/source_candidate_count").cloned().unwrap_or(Value::Null),
        "replay_run_count": driver.pointer("/report/replay_run_count").cloned().unwrap_or(Value::Null),
        "retest_candidate_count": driver.pointer("/report/retest_candidate_count").cloned().unwrap_or(Value::Null),
        "surviving_candidate_count": driver.pointer("/report/surviving_candidate_count").cloned().unwrap_or(Value::Null),
        "shadow_validation_count": driver.pointer("/report/shadow_validation_count").cloned().unwrap_or(Value::Null),
        "paper_trade_candidate_count": driver.pointer("/report/paper_trade_candidate_count").cloned().unwrap_or(Value::Null)
    })
}

fn by_symbol(rows: &[Value]) -> Vec<Value> {
    let mut groups: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    for row in rows {
        let symbol = string_field(row, "primary_symbol")
            .or_else(|| first_symbol(row))
            .unwrap_or("UNKNOWN")
            .to_owned();
        groups.entry(symbol).or_default().push(row.clone());
    }
    groups
        .into_iter()
        .map(|(symbol, mut rows)| {
            rows.sort_by_key(|row| {
                (
                    string_field(row, "candidate_id").unwrap_or("").to_owned(),
                    horizon_rank(string_field(row, "horizon").unwrap_or("")),
                )
            });
            let candidate_ids =
                unique_sorted_strings(rows.iter().filter_map(|row| string_field(row, "candidate_id")));
            let candidates = candidate_ids
                .iter()
                .map(|candidate_id| {
                    let candidate_rows = rows
                        .iter()
                        .filter(|row| string_field(row, "candidate_id") == Some(candidate_id))
                        .cloned()
                        .collect::<Vec<_>>();
                    let first = candidate_rows.first().unwrap_or(&Value::Null);
                    json!({
                        "candidate_id": candidate_id,
                        "candidate_lifecycle_key": string_field(first, "candidate_lifecycle_key"),
                        "symbols": string_array_field(first, "symbols"),
                        "hypothesis_type": string_field(first, "hypothesis_type"),
                        "research_priority": string_field(first, "research_priority"),
                        "horizons": candidate_rows.iter().map(compact_horizon_row).collect::<Vec<_>>()
                    })
                })
                .collect::<Vec<_>>();
            json!({
                "symbol": symbol,
                "candidate_count": candidate_ids.len(),
                "horizon_count": rows.len(),
                "horizons": horizon_counts(&rows),
                "next_action_counts": action_counts(&rows),
                "ready_for_replay_count": count_actions(&rows, &["run_research_replay_for_horizon", "materialize_completed_native_replay_sample"]),
                "waiting_for_market_l1_count": count_action(&rows, "wait_for_market_l1_horizon"),
                "market_l1_coverage_extension_count": count_action(&rows, "extend_market_l1_horizon_coverage"),
                "sample_accumulation_count": count_action(&rows, "accumulate_completed_native_replay_samples"),
                "promotion_ready_for_review_count": count_action(&rows, "promotion_gate_ready_for_review"),
                "candidates": candidates
            })
        })
        .collect()
}

fn by_horizon(rows: &[Value]) -> Vec<Value> {
    horizon_counts(rows)
}

fn horizon_counts(rows: &[Value]) -> Vec<Value> {
    let mut groups: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    for row in rows {
        if let Some(horizon) = string_field(row, "horizon") {
            groups
                .entry(horizon.to_owned())
                .or_default()
                .push(row.clone());
        }
    }
    let mut counts = groups
        .into_iter()
        .map(|(horizon, rows)| {
            json!({
                "horizon": horizon,
                "horizon_count": rows.len(),
                "candidate_count": unique_sorted_strings(rows.iter().filter_map(|row| string_field(row, "candidate_id"))).len(),
                "next_action_counts": action_counts(&rows),
                "waiting_for_market_l1_count": count_action(&rows, "wait_for_market_l1_horizon"),
                "market_l1_coverage_extension_count": count_action(&rows, "extend_market_l1_horizon_coverage"),
                "ready_for_replay_count": count_actions(&rows, &["run_research_replay_for_horizon", "materialize_completed_native_replay_sample"]),
                "sample_accumulation_count": count_action(&rows, "accumulate_completed_native_replay_samples"),
                "promotion_ready_for_review_count": count_action(&rows, "promotion_gate_ready_for_review"),
                "max_completed_sample_deficit": rows.iter().filter_map(|row| i64_field(row, "completed_sample_deficit")).max().unwrap_or(0),
                "max_unseen_window_deficit": rows.iter().filter_map(|row| i64_field(row, "unseen_window_deficit")).max().unwrap_or(0)
            })
        })
        .collect::<Vec<_>>();
    counts.sort_by_key(|value| {
        horizon_rank(value.get("horizon").and_then(Value::as_str).unwrap_or(""))
    });
    counts
}

fn candidate_horizon_matrix(rows: &[Value]) -> Vec<Value> {
    let mut groups: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    for row in rows {
        if let Some(candidate_id) = string_field(row, "candidate_id") {
            groups
                .entry(candidate_id.to_owned())
                .or_default()
                .push(row.clone());
        }
    }
    groups
        .into_iter()
        .map(|(candidate_id, candidate_rows)| {
            let first = candidate_rows.first().unwrap_or(&Value::Null);
            let tracked_horizons = TRACKED_HORIZONS
                .iter()
                .map(|horizon| candidate_horizon_state(&candidate_rows, horizon))
                .collect::<Vec<_>>();
            json!({
                "candidate_id": candidate_id,
                "candidate_lifecycle_key": string_field(first, "candidate_lifecycle_key"),
                "primary_symbol": string_field(first, "primary_symbol").or_else(|| first_symbol(first)),
                "symbols": string_array_field(first, "symbols"),
                "hypothesis_type": string_field(first, "hypothesis_type"),
                "research_priority": string_field(first, "research_priority"),
                "tracked_horizons": tracked_horizons,
                "next_action_counts": action_counts(&tracked_horizons),
                "requested_horizon_count": tracked_horizons.iter().filter(|row| bool_field(row, "requested").unwrap_or(false)).count(),
                "missing_tracked_horizon_count": tracked_horizons.iter().filter(|row| !bool_field(row, "requested").unwrap_or(false)).count(),
                "promotion_ready_horizon_count": tracked_horizons.iter().filter(|row| bool_field(row, "promotion_gate_ready_for_review").unwrap_or(false)).count()
            })
        })
        .collect()
}

fn candidate_horizon_state(rows: &[Value], horizon: &str) -> Value {
    let row = rows
        .iter()
        .find(|row| string_field(row, "horizon") == Some(horizon));
    if let Some(row) = row {
        let next_action = string_field(row, "next_action").unwrap_or("unknown");
        return json!({
            "horizon": horizon,
            "requested": true,
            "next_action": next_action,
            "horizon_market_data_materialized": bool_field(row, "horizon_market_data_materialized").unwrap_or(false),
            "replay_run_count": i64_field(row, "replay_run_count").unwrap_or(0),
            "completed_count": i64_field(row, "completed_count").unwrap_or(0),
            "completed_sample_deficit": row.get("completed_sample_deficit").cloned().unwrap_or(Value::Null),
            "inferred_unseen_window_count": i64_field(row, "inferred_unseen_window_count").unwrap_or(0),
            "unseen_window_deficit": row.get("unseen_window_deficit").cloned().unwrap_or(Value::Null),
            "train_validation_split_materialized": bool_field(row, "train_validation_split_materialized").unwrap_or(false),
            "liquidity_filter_materialized_count": i64_field(row, "liquidity_filter_materialized_count").unwrap_or(0),
            "missing_market_replay_data_count": i64_field(row, "missing_market_replay_data_count").unwrap_or(0),
            "gate_biases": string_array_field(row, "gate_biases"),
            "reason_codes": string_array_field(row, "reason_codes"),
            "promotion_gate_ready_for_review": next_action == "promotion_gate_ready_for_review"
        });
    }
    json!({
        "horizon": horizon,
        "requested": false,
        "next_action": "not_requested",
        "horizon_market_data_materialized": false,
        "replay_run_count": 0,
        "completed_count": 0,
        "completed_sample_deficit": Value::Null,
        "inferred_unseen_window_count": 0,
        "unseen_window_deficit": Value::Null,
        "train_validation_split_materialized": false,
        "liquidity_filter_materialized_count": 0,
        "missing_market_replay_data_count": 0,
        "gate_biases": [],
        "reason_codes": ["horizon_not_requested_by_candidate_bundle"],
        "promotion_gate_ready_for_review": false
    })
}

fn candidate_horizon_matrix_summary(matrix: &[Value]) -> Value {
    let tracked_rows = matrix
        .iter()
        .flat_map(|candidate| {
            candidate
                .get("tracked_horizons")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .cloned()
        .collect::<Vec<_>>();
    json!({
        "tracked_horizons": TRACKED_HORIZONS,
        "candidate_count": matrix.len(),
        "requested_horizon_slot_count": tracked_rows.iter().filter(|row| bool_field(row, "requested").unwrap_or(false)).count(),
        "missing_tracked_horizon_slot_count": tracked_rows.iter().filter(|row| !bool_field(row, "requested").unwrap_or(false)).count(),
        "promotion_ready_horizon_count": tracked_rows.iter().filter(|row| bool_field(row, "promotion_gate_ready_for_review").unwrap_or(false)).count(),
        "next_action_counts": action_counts(&tracked_rows)
    })
}

fn compact_horizon_row(row: &Value) -> Value {
    json!({
        "candidate_id": string_field(row, "candidate_id"),
        "candidate_lifecycle_key": string_field(row, "candidate_lifecycle_key"),
        "primary_symbol": string_field(row, "primary_symbol").or_else(|| first_symbol(row)),
        "symbols": string_array_field(row, "symbols"),
        "hypothesis_type": string_field(row, "hypothesis_type"),
        "research_priority": string_field(row, "research_priority"),
        "horizon": string_field(row, "horizon"),
        "horizon_market_data_materialized": bool_field(row, "horizon_market_data_materialized").unwrap_or(false),
        "replay_run_count": i64_field(row, "replay_run_count").unwrap_or(0),
        "completed_count": i64_field(row, "completed_count").unwrap_or(0),
        "completed_sample_deficit": row.get("completed_sample_deficit").cloned().unwrap_or(Value::Null),
        "inferred_unseen_window_count": i64_field(row, "inferred_unseen_window_count").unwrap_or(0),
        "unseen_window_deficit": row.get("unseen_window_deficit").cloned().unwrap_or(Value::Null),
        "train_validation_split_required": bool_field(row, "train_validation_split_required").unwrap_or(false),
        "train_validation_split_materialized": bool_field(row, "train_validation_split_materialized").unwrap_or(false),
        "liquidity_filter_required": bool_field(row, "liquidity_filter_required").unwrap_or(false),
        "liquidity_filter_materialized_count": i64_field(row, "liquidity_filter_materialized_count").unwrap_or(0),
        "missing_market_replay_data_count": i64_field(row, "missing_market_replay_data_count").unwrap_or(0),
        "gate_biases": string_array_field(row, "gate_biases"),
        "reason_codes": string_array_field(row, "reason_codes"),
        "next_action": string_field(row, "next_action")
    })
}

fn major50_state(latest_universe: &Value, driver: &Value, rows: &[Value]) -> Value {
    let candidate_symbols = unique_sorted_strings(
        rows.iter()
            .filter_map(|row| string_field(row, "primary_symbol").or_else(|| first_symbol(row))),
    );
    let approved_symbols = string_array_pointer(latest_universe, "/approved_symbols");
    let observed_symbols = string_array_pointer(latest_universe, "/observed_symbols");
    let eligible_candidate_symbols =
        string_array_pointer(driver, "/manifest/eligible_candidate_symbols")
            .into_iter()
            .collect::<BTreeSet<_>>();
    let eligible_candidate_symbols = if eligible_candidate_symbols.is_empty() {
        candidate_symbols.iter().cloned().collect::<BTreeSet<_>>()
    } else {
        eligible_candidate_symbols
    };
    let approved_symbol_set = approved_symbols.iter().cloned().collect::<BTreeSet<_>>();
    let candidate_symbol_set = candidate_symbols.iter().cloned().collect::<BTreeSet<_>>();
    let candidate_symbols_in_approved_universe =
        intersection_sorted(&candidate_symbol_set, &approved_symbol_set);
    let eligible_candidate_symbols_in_approved_universe =
        intersection_sorted(&eligible_candidate_symbols, &approved_symbol_set);
    let approved_symbols_without_selected_candidate =
        difference_sorted(&approved_symbol_set, &candidate_symbol_set);
    let approved_symbols_without_eligible_candidate =
        difference_sorted(&approved_symbol_set, &eligible_candidate_symbols);
    json!({
        "universe_mode": driver.pointer("/manifest/universe_mode").cloned().unwrap_or(Value::Null),
        "latest_universe_present": latest_universe.get("present").cloned().unwrap_or(Value::Null),
        "observed_symbol_count": latest_universe.get("observed_symbol_count").and_then(Value::as_u64).unwrap_or(observed_symbols.len() as u64),
        "approved_symbol_count": latest_universe.get("approved_symbol_count").and_then(Value::as_u64).unwrap_or(approved_symbols.len() as u64),
        "excluded_symbol_count": latest_universe.get("excluded_symbol_count").cloned().unwrap_or(Value::Null),
        "candidate_symbol_count": candidate_symbols.len(),
        "candidate_symbols": candidate_symbols,
        "eligible_candidate_pool_count": driver.pointer("/manifest/eligible_candidate_pool_count").cloned().unwrap_or(Value::Null),
        "selected_candidate_limit_reached": driver.pointer("/manifest/selected_candidate_limit_reached").cloned().unwrap_or(Value::Null),
        "unselected_eligible_candidate_count": driver.pointer("/manifest/unselected_eligible_candidate_count").cloned().unwrap_or(Value::Null),
        "eligible_candidate_symbols": eligible_candidate_symbols.iter().cloned().collect::<Vec<_>>(),
        "unselected_eligible_candidate_symbols": string_array_pointer(driver, "/manifest/unselected_eligible_candidate_symbols"),
        "candidate_symbols_in_approved_universe": candidate_symbols_in_approved_universe,
        "eligible_candidate_symbols_in_approved_universe": eligible_candidate_symbols_in_approved_universe,
        "approved_symbols_without_selected_candidate": approved_symbols_without_selected_candidate,
        "approved_symbols_without_eligible_candidate": approved_symbols_without_eligible_candidate,
        "selected_symbols_not_in_approved_universe": if approved_symbols.is_empty() { Vec::<String>::new() } else { difference_sorted(&candidate_symbol_set, &approved_symbol_set) },
        "candidate_symbol_coverage_of_approved_universe": coverage(candidate_symbols_in_approved_universe_len(latest_universe, rows), approved_symbols.len()),
        "eligible_candidate_symbol_coverage_of_approved_universe": coverage(eligible_candidate_symbols_in_approved_universe_len(latest_universe, driver, rows), approved_symbols.len())
    })
}

fn research_factory_progression(
    latest_universe: &Value,
    rows: &[Value],
    promotion_passed: bool,
    shadow_created: bool,
    paper_created: bool,
) -> Value {
    let candidate_symbols = unique_sorted_strings(
        rows.iter()
            .filter_map(|row| string_field(row, "primary_symbol").or_else(|| first_symbol(row))),
    );
    let candidate_ids = unique_sorted_strings(
        rows.iter()
            .filter_map(|row| string_field(row, "candidate_id")),
    );
    let research_replayed_symbols = unique_sorted_strings(rows.iter().filter_map(|row| {
        (i64_field(row, "replay_run_count").unwrap_or(0) > 0)
            .then(|| string_field(row, "primary_symbol").or_else(|| first_symbol(row)))
            .flatten()
    }));
    let research_replayed_candidate_ids = unique_sorted_strings(rows.iter().filter_map(|row| {
        (i64_field(row, "replay_run_count").unwrap_or(0) > 0)
            .then(|| string_field(row, "candidate_id"))
            .flatten()
    }));
    let promotion_ready_symbols = unique_sorted_strings(rows.iter().filter_map(|row| {
        (string_field(row, "next_action") == Some("promotion_gate_ready_for_review"))
            .then(|| string_field(row, "primary_symbol").or_else(|| first_symbol(row)))
            .flatten()
    }));
    let promotion_ready_candidate_ids = unique_sorted_strings(rows.iter().filter_map(|row| {
        (string_field(row, "next_action") == Some("promotion_gate_ready_for_review"))
            .then(|| string_field(row, "candidate_id"))
            .flatten()
    }));
    let promoted_symbols = if promotion_passed {
        promotion_ready_symbols.clone()
    } else {
        rows_with_promote_bias_symbols(rows)
    };
    let promoted_candidate_ids = if promotion_passed {
        promotion_ready_candidate_ids.clone()
    } else {
        rows_with_promote_bias_candidate_ids(rows)
    };
    json!({
        "major50_observed_symbol_count": latest_universe.get("observed_symbol_count").and_then(Value::as_u64).unwrap_or_else(|| string_array_pointer(latest_universe, "/observed_symbols").len() as u64),
        "major50_approved_symbol_count": latest_universe.get("approved_symbol_count").and_then(Value::as_u64).unwrap_or_else(|| string_array_pointer(latest_universe, "/approved_symbols").len() as u64),
        "candidate_generated_symbol_count": candidate_symbols.len(),
        "candidate_generated_candidate_count": candidate_ids.len(),
        "research_replayed_symbol_count": research_replayed_symbols.len(),
        "research_replayed_candidate_count": research_replayed_candidate_ids.len(),
        "promotion_ready_symbol_count": promotion_ready_symbols.len(),
        "promotion_ready_candidate_count": promotion_ready_candidate_ids.len(),
        "promoted_symbol_count": promoted_symbols.len(),
        "promoted_candidate_count": promoted_candidate_ids.len(),
        "shadow_created": shadow_created,
        "paper_created": paper_created,
        "live_enabled": false,
        "symbols": {
            "candidate_generated": candidate_symbols,
            "research_replayed": research_replayed_symbols,
            "promotion_ready": promotion_ready_symbols,
            "promoted": promoted_symbols
        },
        "candidates": {
            "candidate_generated": candidate_ids,
            "research_replayed": research_replayed_candidate_ids,
            "promotion_ready": promotion_ready_candidate_ids,
            "promoted": promoted_candidate_ids
        }
    })
}

fn coverage_gaps(
    latest_universe: &Value,
    driver: &Value,
    rows: &[Value],
    shadow_created: bool,
) -> Value {
    let approved_symbols = string_array_pointer(latest_universe, "/approved_symbols")
        .into_iter()
        .collect::<BTreeSet<_>>();
    let candidate_symbols = unique_sorted_strings(
        rows.iter()
            .filter_map(|row| string_field(row, "primary_symbol").or_else(|| first_symbol(row))),
    )
    .into_iter()
    .collect::<BTreeSet<_>>();
    let eligible_symbols = {
        let symbols = string_array_pointer(driver, "/manifest/eligible_candidate_symbols");
        if symbols.is_empty() {
            candidate_symbols.clone()
        } else {
            symbols.into_iter().collect()
        }
    };
    let candidate_ids = unique_sorted_strings(
        rows.iter()
            .filter_map(|row| string_field(row, "candidate_id")),
    )
    .into_iter()
    .collect::<BTreeSet<_>>();
    let research_replayed_candidate_ids = unique_sorted_strings(rows.iter().filter_map(|row| {
        (i64_field(row, "replay_run_count").unwrap_or(0) > 0)
            .then(|| string_field(row, "candidate_id"))
            .flatten()
    }))
    .into_iter()
    .collect::<BTreeSet<_>>();
    let research_replayed_symbols = unique_sorted_strings(rows.iter().filter_map(|row| {
        (i64_field(row, "replay_run_count").unwrap_or(0) > 0)
            .then(|| string_field(row, "primary_symbol").or_else(|| first_symbol(row)))
            .flatten()
    }))
    .into_iter()
    .collect::<BTreeSet<_>>();
    let promotion_ready_candidate_ids = unique_sorted_strings(rows.iter().filter_map(|row| {
        (string_field(row, "next_action") == Some("promotion_gate_ready_for_review"))
            .then(|| string_field(row, "candidate_id"))
            .flatten()
    }))
    .into_iter()
    .collect::<BTreeSet<_>>();
    let promotion_ready_symbols = unique_sorted_strings(rows.iter().filter_map(|row| {
        (string_field(row, "next_action") == Some("promotion_gate_ready_for_review"))
            .then(|| string_field(row, "primary_symbol").or_else(|| first_symbol(row)))
            .flatten()
    }))
    .into_iter()
    .collect::<BTreeSet<_>>();
    let promoted_candidate_ids = rows_with_promote_bias_candidate_ids(rows)
        .into_iter()
        .collect::<BTreeSet<_>>();
    let promoted_symbols = rows_with_promote_bias_symbols(rows)
        .into_iter()
        .collect::<BTreeSet<_>>();
    json!({
        "approved_symbols_without_candidate": difference_sorted(&approved_symbols, &eligible_symbols),
        "approved_symbols_without_selected_candidate": difference_sorted(&approved_symbols, &candidate_symbols),
        "approved_symbols_without_eligible_candidate": difference_sorted(&approved_symbols, &eligible_symbols),
        "unselected_eligible_candidate_symbols": string_array_pointer(driver, "/manifest/unselected_eligible_candidate_symbols"),
        "candidate_symbols_without_replay": difference_sorted(&candidate_symbols, &research_replayed_symbols),
        "candidate_ids_without_replay": difference_sorted(&candidate_ids, &research_replayed_candidate_ids),
        "replayed_symbols_without_promotion_ready": difference_sorted(&research_replayed_symbols, &promotion_ready_symbols),
        "replayed_symbols_without_promotion": difference_sorted(&research_replayed_symbols, &promoted_symbols),
        "replayed_candidate_ids_without_promotion_ready": difference_sorted(&research_replayed_candidate_ids, &promotion_ready_candidate_ids),
        "replayed_candidate_ids_without_promotion": difference_sorted(&research_replayed_candidate_ids, &promoted_candidate_ids),
        "promotion_ready_symbols_without_shadow": if shadow_created { Vec::<String>::new() } else { promotion_ready_symbols.iter().cloned().collect::<Vec<_>>() },
        "promotion_ready_candidate_ids_without_shadow": if shadow_created { Vec::<String>::new() } else { promotion_ready_candidate_ids.iter().cloned().collect::<Vec<_>>() },
        "promoted_symbols_without_shadow": if shadow_created { Vec::<String>::new() } else { promoted_symbols.iter().cloned().collect::<Vec<_>>() },
        "promoted_candidate_ids_without_shadow": if shadow_created { Vec::<String>::new() } else { promoted_candidate_ids.iter().cloned().collect::<Vec<_>>() }
    })
}

fn research_factory_gap_summary(
    latest_universe: &Value,
    driver: &Value,
    rows: &[Value],
    promotion_passed: bool,
    shadow_created: bool,
    paper_created: bool,
    safe_next_actions: &[String],
) -> Value {
    let gaps = coverage_gaps(latest_universe, driver, rows, shadow_created);
    let blocking_stage = if gaps
        .get("approved_symbols_without_eligible_candidate")
        .and_then(Value::as_array)
        .map(|values| !values.is_empty())
        .unwrap_or(false)
    {
        "candidate_generation_coverage"
    } else if gaps
        .get("candidate_ids_without_replay")
        .and_then(Value::as_array)
        .map(|values| !values.is_empty())
        .unwrap_or(false)
    {
        "research_replay_coverage"
    } else if gaps
        .get("promotion_ready_symbols_without_shadow")
        .and_then(Value::as_array)
        .map(|values| !values.is_empty())
        .unwrap_or(false)
    {
        "shadow_review_gate"
    } else if !promotion_passed {
        "promotion_evidence"
    } else if !paper_created {
        "paper_validation_gate"
    } else {
        "human_live_approval_boundary"
    };
    json!({
        "blocking_stage": blocking_stage,
        "stage_counts": {
            "major50_observed": latest_universe.get("observed_symbol_count").and_then(Value::as_u64).unwrap_or_else(|| string_array_pointer(latest_universe, "/observed_symbols").len() as u64),
            "major50_approved": latest_universe.get("approved_symbol_count").and_then(Value::as_u64).unwrap_or_else(|| string_array_pointer(latest_universe, "/approved_symbols").len() as u64),
            "candidate_generated": unique_sorted_strings(rows.iter().filter_map(|row| string_field(row, "primary_symbol").or_else(|| first_symbol(row)))).len(),
            "candidate_generated_candidates": unique_sorted_strings(rows.iter().filter_map(|row| string_field(row, "candidate_id"))).len(),
            "research_replayed": unique_sorted_strings(rows.iter().filter_map(|row| (i64_field(row, "replay_run_count").unwrap_or(0) > 0).then(|| string_field(row, "primary_symbol").or_else(|| first_symbol(row))).flatten())).len(),
            "research_replayed_candidates": unique_sorted_strings(rows.iter().filter_map(|row| (i64_field(row, "replay_run_count").unwrap_or(0) > 0).then(|| string_field(row, "candidate_id")).flatten())).len(),
            "promotion_ready": unique_sorted_strings(rows.iter().filter_map(|row| (string_field(row, "next_action") == Some("promotion_gate_ready_for_review")).then(|| string_field(row, "primary_symbol").or_else(|| first_symbol(row))).flatten())).len(),
            "promotion_ready_candidates": unique_sorted_strings(rows.iter().filter_map(|row| (string_field(row, "next_action") == Some("promotion_gate_ready_for_review")).then(|| string_field(row, "candidate_id")).flatten())).len(),
            "promoted": rows_with_promote_bias_symbols(rows).len(),
            "promoted_candidates": rows_with_promote_bias_candidate_ids(rows).len()
        },
        "gap_counts": {
            "approved_symbols_without_candidate": gaps.get("approved_symbols_without_candidate").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
            "approved_symbols_without_selected_candidate": gaps.get("approved_symbols_without_selected_candidate").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
            "approved_symbols_without_eligible_candidate": gaps.get("approved_symbols_without_eligible_candidate").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
            "unselected_eligible_candidate_symbols": gaps.get("unselected_eligible_candidate_symbols").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
            "candidate_symbols_without_replay": gaps.get("candidate_symbols_without_replay").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
            "candidate_ids_without_replay": gaps.get("candidate_ids_without_replay").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
            "replayed_symbols_without_promotion_ready": gaps.get("replayed_symbols_without_promotion_ready").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
            "replayed_symbols_without_promotion": gaps.get("replayed_symbols_without_promotion").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
            "replayed_candidate_ids_without_promotion_ready": gaps.get("replayed_candidate_ids_without_promotion_ready").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
            "replayed_candidate_ids_without_promotion": gaps.get("replayed_candidate_ids_without_promotion").and_then(Value::as_array).map(Vec::len).unwrap_or(0)
        },
        "safe_next_actions": safe_next_actions,
        "shadow_created": shadow_created,
        "paper_created": paper_created,
        "live_enabled": false
    })
}

fn next_decision_verdict(
    promotion_passed: bool,
    promotion_ready_for_review_count: usize,
    market_l1_coverage_extension_count: usize,
    ready_for_replay_count: usize,
    waiting_for_market_l1_count: usize,
    sample_accumulation_count: usize,
) -> &'static str {
    if promotion_passed {
        "PROMOTE_PRESENT_REVIEW_BEFORE_SHADOW"
    } else if promotion_ready_for_review_count > 0 {
        "PROMOTION_GATE_READY_FOR_REVIEW"
    } else if market_l1_coverage_extension_count > 0 {
        "EXTEND_MARKET_L1_HORIZON_COVERAGE"
    } else if ready_for_replay_count > 0 {
        "REPLAY_READY_FOR_SOME_HORIZONS"
    } else if waiting_for_market_l1_count > 0 {
        "WAIT_FOR_MARKET_L1_HORIZON"
    } else if sample_accumulation_count > 0 {
        "ACCUMULATE_COMPLETED_NATIVE_REPLAY_SAMPLES"
    } else {
        "INSPECT_REMAINING_GATE_REASONS"
    }
}

fn safe_next_actions(
    promotion_passed: bool,
    promotion_ready_for_review_count: usize,
    market_l1_coverage_extension_count: usize,
    ready_for_replay_count: usize,
    waiting_for_market_l1_count: usize,
    sample_accumulation_count: usize,
) -> Vec<String> {
    let mut actions = Vec::new();
    if promotion_passed {
        actions.push("review_promoted_candidates_before_shadow".to_owned());
    }
    if promotion_ready_for_review_count > 0 {
        actions.push("review_promotion_gate_ready_horizons".to_owned());
    }
    if market_l1_coverage_extension_count > 0 {
        actions.push("extend_market_l1_horizon_coverage".to_owned());
    }
    if ready_for_replay_count > 0 {
        actions.push("rerun_current_approved_research_batch_after_market_l1_advances".to_owned());
    }
    if waiting_for_market_l1_count > 0 {
        actions.push("wait_for_market_l1_horizon_materialization".to_owned());
    }
    if sample_accumulation_count > 0 {
        actions.push("keep_accumulating_completed_native_replay_samples".to_owned());
    }
    actions.sort();
    actions.dedup();
    actions
}

fn blocked_actions(promotion_passed: bool, shadow_created: bool) -> Vec<String> {
    let mut actions = Vec::new();
    if !promotion_passed {
        actions.push("do_not_create_shadow_without_promotion".to_owned());
    }
    if !shadow_created {
        actions.push("do_not_create_paper_without_passed_shadow".to_owned());
    }
    actions.push("do_not_enable_live_from_research_batch".to_owned());
    actions
}

fn action_counts(rows: &[Value]) -> Vec<Value> {
    let mut counts = BTreeMap::<String, usize>::new();
    for row in rows {
        if let Some(action) = string_field(row, "next_action") {
            *counts.entry(action.to_owned()).or_default() += 1;
        }
    }
    let mut values = counts
        .into_iter()
        .map(|(next_action, count)| json!({ "next_action": next_action, "count": count }))
        .collect::<Vec<_>>();
    values.sort_by(|left, right| {
        right
            .get("count")
            .and_then(Value::as_u64)
            .cmp(&left.get("count").and_then(Value::as_u64))
            .then_with(|| {
                left.get("next_action")
                    .and_then(Value::as_str)
                    .cmp(&right.get("next_action").and_then(Value::as_str))
            })
    });
    values
}

fn count_action(rows: &[Value], action: &str) -> usize {
    rows.iter()
        .filter(|row| string_field(row, "next_action") == Some(action))
        .count()
}

fn count_actions(rows: &[Value], actions: &[&str]) -> usize {
    rows.iter()
        .filter(|row| {
            string_field(row, "next_action")
                .map(|action| actions.contains(&action))
                .unwrap_or(false)
        })
        .count()
}

fn min_ms_for_action(rows: &[Value], action: &str, field: &str) -> Option<i64> {
    rows.iter()
        .filter(|row| string_field(row, "next_action") == Some(action))
        .filter_map(|row| i64_field(row, field))
        .min()
}

fn max_ms_for_action(rows: &[Value], action: &str, field: &str) -> Option<i64> {
    rows.iter()
        .filter(|row| string_field(row, "next_action") == Some(action))
        .filter_map(|row| i64_field(row, field))
        .max()
}

fn rows_with_promote_bias_symbols(rows: &[Value]) -> Vec<String> {
    unique_sorted_strings(rows.iter().filter_map(|row| {
        has_promote_bias(row)
            .then(|| string_field(row, "primary_symbol").or_else(|| first_symbol(row)))
            .flatten()
    }))
}

fn rows_with_promote_bias_candidate_ids(rows: &[Value]) -> Vec<String> {
    unique_sorted_strings(rows.iter().filter_map(|row| {
        has_promote_bias(row)
            .then(|| string_field(row, "candidate_id"))
            .flatten()
    }))
}

fn has_promote_bias(row: &Value) -> bool {
    row.get("gate_biases")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .any(|bias| bias.starts_with("PROMOTE"))
}

fn string_field<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value.get(field).and_then(Value::as_str)
}

fn i64_field(value: &Value, field: &str) -> Option<i64> {
    value.get(field).and_then(Value::as_i64)
}

fn bool_field(value: &Value, field: &str) -> Option<bool> {
    value.get(field).and_then(Value::as_bool)
}

fn bool_pointer(value: &Value, pointer: &str) -> Option<bool> {
    value.pointer(pointer).and_then(Value::as_bool)
}

fn first_symbol(value: &Value) -> Option<&str> {
    value
        .get("symbols")
        .and_then(Value::as_array)
        .and_then(|symbols| symbols.first())
        .and_then(Value::as_str)
}

fn string_array_field(value: &Value, field: &str) -> Vec<String> {
    value
        .get(field)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(ToOwned::to_owned)
        .collect()
}

fn string_array_pointer(value: &Value, pointer: &str) -> Vec<String> {
    value
        .pointer(pointer)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(ToOwned::to_owned)
        .collect()
}

fn unique_sorted_strings<'a>(values: impl Iterator<Item = &'a str>) -> Vec<String> {
    values
        .map(ToOwned::to_owned)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn intersection_sorted(left: &BTreeSet<String>, right: &BTreeSet<String>) -> Vec<String> {
    left.intersection(right).cloned().collect()
}

fn difference_sorted(left: &BTreeSet<String>, right: &BTreeSet<String>) -> Vec<String> {
    left.difference(right).cloned().collect()
}

fn candidate_symbols_in_approved_universe_len(latest_universe: &Value, rows: &[Value]) -> usize {
    let approved_symbols = string_array_pointer(latest_universe, "/approved_symbols")
        .into_iter()
        .collect::<BTreeSet<_>>();
    let candidate_symbols = unique_sorted_strings(
        rows.iter()
            .filter_map(|row| string_field(row, "primary_symbol").or_else(|| first_symbol(row))),
    )
    .into_iter()
    .collect::<BTreeSet<_>>();
    intersection_sorted(&candidate_symbols, &approved_symbols).len()
}

fn eligible_candidate_symbols_in_approved_universe_len(
    latest_universe: &Value,
    driver: &Value,
    rows: &[Value],
) -> usize {
    let approved_symbols = string_array_pointer(latest_universe, "/approved_symbols")
        .into_iter()
        .collect::<BTreeSet<_>>();
    let eligible_symbols = {
        let symbols = string_array_pointer(driver, "/manifest/eligible_candidate_symbols");
        if symbols.is_empty() {
            unique_sorted_strings(rows.iter().filter_map(|row| {
                string_field(row, "primary_symbol").or_else(|| first_symbol(row))
            }))
        } else {
            symbols
        }
    }
    .into_iter()
    .collect::<BTreeSet<_>>();
    intersection_sorted(&eligible_symbols, &approved_symbols).len()
}

fn coverage(numerator: usize, denominator: usize) -> Option<f64> {
    (denominator > 0).then(|| numerator as f64 / denominator as f64)
}

fn horizon_rank(horizon: &str) -> usize {
    match horizon {
        "1h" => 1,
        "4h" => 2,
        "24h" | "1d" => 3,
        "72h" => 4,
        "7d" => 5,
        _ => 99,
    }
}

fn iso8601_ms(ms: i64) -> String {
    let secs = ms.div_euclid(1000);
    DateTime::<Utc>::from_timestamp(secs, 0)
        .unwrap_or(DateTime::<Utc>::UNIX_EPOCH)
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan() -> Value {
        json!({
            "schema_version": "research_retest_horizon_plan_v1",
            "latest_l1_as_of_ms": 1_779_710_400_000_i64,
            "horizon_rows": [
                {
                    "candidate_id": "cand_a",
                    "candidate_lifecycle_key": "cand_a:v1",
                    "symbols": ["AAVE"],
                    "primary_symbol": "AAVE",
                    "hypothesis_type": "event_reaction",
                    "research_priority": "p0",
                    "horizon": "1h",
                    "horizon_due_ms": 1_779_710_300_000_i64,
                    "horizon_market_data_materialized": true,
                    "replay_run_count": 2,
                    "completed_count": 1,
                    "completed_sample_deficit": 29,
                    "inferred_unseen_window_count": 1,
                    "unseen_window_deficit": 19,
                    "train_validation_split_materialized": true,
                    "liquidity_filter_materialized_count": 1,
                    "missing_market_replay_data_count": 0,
                    "gate_biases": ["RETEST_BIAS"],
                    "reason_codes": ["sample_deficit"],
                    "next_action": "accumulate_completed_native_replay_samples"
                },
                {
                    "candidate_id": "cand_a",
                    "candidate_lifecycle_key": "cand_a:v1",
                    "symbols": ["AAVE"],
                    "primary_symbol": "AAVE",
                    "hypothesis_type": "event_reaction",
                    "research_priority": "p0",
                    "horizon": "4h",
                    "horizon_due_ms": 1_779_719_361_452_i64,
                    "horizon_market_data_materialized": false,
                    "replay_run_count": 2,
                    "completed_count": 0,
                    "completed_sample_deficit": 30,
                    "inferred_unseen_window_count": 1,
                    "unseen_window_deficit": 19,
                    "reason_codes": ["waiting_for_l1"],
                    "next_action": "wait_for_market_l1_horizon"
                }
            ]
        })
    }

    #[test]
    fn builds_run_status_when_some_horizons_can_accumulate_samples() {
        let status = build_retest_horizon_status(
            &plan(),
            None,
            &RetestHorizonStatusBuildOptions {
                generated_at_ms: 1_779_714_000_000,
                plan_file: Some("/tmp/plan.json".to_owned()),
                driver_summary_file: None,
                checkpoint_s3_write: false,
            },
        )
        .expect("status builds");

        assert_eq!(
            status["schema_version"],
            json!(RETEST_HORIZON_STATUS_SCHEMA_VERSION)
        );
        assert_eq!(status["verdict"], json!("REPLAY_READY_FOR_SOME_HORIZONS"));
        assert_eq!(
            status["next_decision"]["scheduler_hint"]["run_now_replay_ready"],
            json!(true)
        );
        assert_eq!(
            status["next_decision"]["scheduler_hint"]["run_research_after_l1_as_of_ms"],
            json!(1_779_719_361_452_i64)
        );
        assert_eq!(status["by_symbol"][0]["symbol"], json!("AAVE"));
        assert_eq!(
            status["by_symbol"][0]["candidates"][0]["horizons"][1]["next_action"],
            json!("wait_for_market_l1_horizon")
        );
    }

    #[test]
    fn rejects_plan_without_rows() {
        let error = build_retest_horizon_status(
            &json!({"schema_version": "research_retest_horizon_plan_v1"}),
            None,
            &RetestHorizonStatusBuildOptions {
                generated_at_ms: 1,
                plan_file: None,
                driver_summary_file: None,
                checkpoint_s3_write: false,
            },
        )
        .expect_err("rows are required");
        assert!(error.to_string().contains("horizon_rows"));
    }
}
