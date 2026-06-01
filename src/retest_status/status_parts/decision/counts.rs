use serde_json::{Value, json};
use std::collections::BTreeMap;

use super::super::fields::{i64_field, string_field};

pub(in crate::retest_status) fn action_counts(rows: &[Value]) -> Vec<Value> {
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

pub(in crate::retest_status) fn count_action(rows: &[Value], action: &str) -> usize {
    rows.iter()
        .filter(|row| string_field(row, "next_action") == Some(action))
        .count()
}

pub(in crate::retest_status) fn count_actions(rows: &[Value], actions: &[&str]) -> usize {
    rows.iter()
        .filter(|row| {
            string_field(row, "next_action")
                .map(|action| actions.contains(&action))
                .unwrap_or(false)
        })
        .count()
}

pub(in crate::retest_status) fn min_ms_for_action(
    rows: &[Value],
    action: &str,
    field: &str,
) -> Option<i64> {
    rows.iter()
        .filter(|row| string_field(row, "next_action") == Some(action))
        .filter_map(|row| i64_field(row, field))
        .min()
}

pub(in crate::retest_status) fn max_ms_for_action(
    rows: &[Value],
    action: &str,
    field: &str,
) -> Option<i64> {
    rows.iter()
        .filter(|row| string_field(row, "next_action") == Some(action))
        .filter_map(|row| i64_field(row, field))
        .max()
}
