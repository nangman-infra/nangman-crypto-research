use serde_json::{Value, json};
use std::collections::BTreeMap;

use super::row::candidate_horizon_state;
use crate::retest_status::status_parts::decision::action_counts;
use crate::retest_status::status_parts::fields::{
    bool_field, first_symbol, string_array_field, string_field,
};

pub(in crate::retest_status) fn candidate_horizon_matrix(rows: &[Value]) -> Vec<Value> {
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
            let tracked_horizons = super::super::super::TRACKED_HORIZONS
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

pub(in crate::retest_status) fn candidate_horizon_matrix_summary(matrix: &[Value]) -> Value {
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
        "tracked_horizons": super::super::super::TRACKED_HORIZONS,
        "candidate_count": matrix.len(),
        "requested_horizon_slot_count": tracked_rows.iter().filter(|row| bool_field(row, "requested").unwrap_or(false)).count(),
        "missing_tracked_horizon_slot_count": tracked_rows.iter().filter(|row| !bool_field(row, "requested").unwrap_or(false)).count(),
        "promotion_ready_horizon_count": tracked_rows.iter().filter(|row| bool_field(row, "promotion_gate_ready_for_review").unwrap_or(false)).count(),
        "next_action_counts": action_counts(&tracked_rows)
    })
}
