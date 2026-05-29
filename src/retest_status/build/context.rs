use serde_json::Value;

use super::super::status_parts::{i64_field, string_field, unique_sorted_strings};

pub(super) struct StatusBuildContext {
    pub(super) rows: Vec<Value>,
    pub(super) driver: Value,
    pub(super) latest_universe: Value,
    pub(super) latest_l1_as_of_ms: Option<i64>,
    pub(super) candidate_ids: Vec<String>,
    pub(super) replayed_candidate_ids: Vec<String>,
}

pub(super) fn prepare_context(plan: &Value, driver_summary: Option<&Value>) -> StatusBuildContext {
    let rows = plan
        .get("horizon_rows")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let driver = driver_summary.cloned().unwrap_or(Value::Null);
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

    StatusBuildContext {
        rows,
        driver,
        latest_universe,
        latest_l1_as_of_ms,
        candidate_ids,
        replayed_candidate_ids,
    }
}
