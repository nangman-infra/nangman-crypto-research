use super::FocusedRetestRow;
use super::fields::{integer_field, string_array_field, string_field};
use super::filters::FocusRowFilters;
use serde_json::Value;

pub(super) fn candidate_rows(
    symbol: &str,
    candidate: &Value,
    filters: &FocusRowFilters<'_>,
) -> Vec<FocusedRetestRow> {
    let Some(candidate_id) = string_field(candidate, "candidate_id") else {
        return Vec::new();
    };
    let candidate_lifecycle_key = string_field(candidate, "candidate_lifecycle_key");
    if !filters.allows_lifecycle_key(candidate_lifecycle_key) {
        return Vec::new();
    }
    let Some(horizons) = candidate.get("horizons").and_then(Value::as_array) else {
        return Vec::new();
    };
    horizons
        .iter()
        .filter_map(|horizon| {
            horizon_row(
                symbol,
                candidate,
                candidate_id,
                candidate_lifecycle_key,
                horizon,
                filters,
            )
        })
        .collect()
}

fn horizon_row(
    symbol: &str,
    candidate: &Value,
    candidate_id: &str,
    candidate_lifecycle_key: Option<&str>,
    horizon: &Value,
    filters: &FocusRowFilters<'_>,
) -> Option<FocusedRetestRow> {
    let next_action = string_field(horizon, "next_action")?;
    if !filters.allows_action(next_action) {
        return None;
    }
    Some(FocusedRetestRow {
        candidate_id: candidate_id.to_owned(),
        candidate_lifecycle_key: candidate_lifecycle_key.map(ToOwned::to_owned),
        symbol: symbol.to_owned(),
        symbols: string_array_field(horizon, "symbols"),
        hypothesis_type: string_field(candidate, "hypothesis_type").map(ToOwned::to_owned),
        research_priority: string_field(candidate, "research_priority").map(ToOwned::to_owned),
        horizon: string_field(horizon, "horizon")
            .unwrap_or("unknown")
            .to_owned(),
        next_action: next_action.to_owned(),
        replay_run_count: integer_field(horizon, "replay_run_count"),
        completed_count: integer_field(horizon, "completed_count"),
        completed_sample_deficit: integer_field(horizon, "completed_sample_deficit"),
        inferred_unseen_window_count: integer_field(horizon, "inferred_unseen_window_count"),
        unseen_window_deficit: integer_field(horizon, "unseen_window_deficit"),
        reason_codes: string_array_field(horizon, "reason_codes"),
    })
}
