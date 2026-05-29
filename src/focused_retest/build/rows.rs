use super::super::types::FocusedRetestRow;
use crate::error::AppResult;
use serde_json::Value;
use std::collections::BTreeSet;

pub(super) fn focus_rows(
    status: &Value,
    actions: &[String],
    candidate_lifecycle_key_filter: &[String],
) -> AppResult<Vec<FocusedRetestRow>> {
    let action_set = actions.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let lifecycle_filter = candidate_lifecycle_key_filter
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut rows = Vec::new();
    let Some(symbols) = status.get("by_symbol").and_then(Value::as_array) else {
        return Ok(rows);
    };
    for symbol_doc in symbols {
        let symbol = string_field(symbol_doc, "symbol").unwrap_or("UNKNOWN");
        let Some(candidates) = symbol_doc.get("candidates").and_then(Value::as_array) else {
            continue;
        };
        for candidate in candidates {
            let Some(candidate_id) = string_field(candidate, "candidate_id") else {
                continue;
            };
            let candidate_lifecycle_key = string_field(candidate, "candidate_lifecycle_key");
            if !lifecycle_filter.is_empty()
                && !candidate_lifecycle_key.is_some_and(|key| lifecycle_filter.contains(key))
            {
                continue;
            }
            let Some(horizons) = candidate.get("horizons").and_then(Value::as_array) else {
                continue;
            };
            for horizon in horizons {
                let Some(next_action) = string_field(horizon, "next_action") else {
                    continue;
                };
                if !action_set.contains(next_action) {
                    continue;
                }
                rows.push(FocusedRetestRow {
                    candidate_id: candidate_id.to_owned(),
                    candidate_lifecycle_key: candidate_lifecycle_key.map(ToOwned::to_owned),
                    symbol: symbol.to_owned(),
                    symbols: string_array_field(horizon, "symbols"),
                    hypothesis_type: string_field(candidate, "hypothesis_type")
                        .map(ToOwned::to_owned),
                    research_priority: string_field(candidate, "research_priority")
                        .map(ToOwned::to_owned),
                    horizon: string_field(horizon, "horizon")
                        .unwrap_or("unknown")
                        .to_owned(),
                    next_action: next_action.to_owned(),
                    replay_run_count: integer_field(horizon, "replay_run_count"),
                    completed_count: integer_field(horizon, "completed_count"),
                    completed_sample_deficit: integer_field(horizon, "completed_sample_deficit"),
                    inferred_unseen_window_count: integer_field(
                        horizon,
                        "inferred_unseen_window_count",
                    ),
                    unseen_window_deficit: integer_field(horizon, "unseen_window_deficit"),
                    reason_codes: string_array_field(horizon, "reason_codes"),
                });
            }
        }
    }
    rows.sort_by(|left, right| {
        (
            left.symbol.as_str(),
            left.candidate_id.as_str(),
            horizon_order(&left.horizon),
        )
            .cmp(&(
                right.symbol.as_str(),
                right.candidate_id.as_str(),
                horizon_order(&right.horizon),
            ))
    });
    Ok(rows)
}

pub(super) fn horizon_order(horizon: &str) -> u8 {
    match horizon {
        "1h" => 1,
        "4h" => 2,
        "24h" | "1d" => 3,
        "72h" => 4,
        "7d" => 5,
        _ => 99,
    }
}

fn string_field<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn integer_field(value: &Value, field: &str) -> Option<i64> {
    value.get(field).and_then(Value::as_i64)
}

fn string_array_field(value: &Value, field: &str) -> Vec<String> {
    value
        .get(field)
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}
