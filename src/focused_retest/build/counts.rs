use super::super::types::{FocusedRetestActionCount, FocusedRetestHorizonCount, FocusedRetestRow};
use super::rows::horizon_order;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn action_counts(rows: &[FocusedRetestRow]) -> Vec<FocusedRetestActionCount> {
    let mut counts = BTreeMap::<String, usize>::new();
    for row in rows {
        *counts.entry(row.next_action.clone()).or_default() += 1;
    }
    let mut counts = counts
        .into_iter()
        .map(|(next_action, count)| FocusedRetestActionCount { next_action, count })
        .collect::<Vec<_>>();
    counts.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.next_action.cmp(&right.next_action))
    });
    counts
}

pub(super) fn horizon_counts(rows: &[FocusedRetestRow]) -> Vec<FocusedRetestHorizonCount> {
    let mut counts = BTreeMap::<String, usize>::new();
    for row in rows {
        *counts.entry(row.horizon.clone()).or_default() += 1;
    }
    let mut counts = counts
        .into_iter()
        .map(|(horizon, count)| FocusedRetestHorizonCount { horizon, count })
        .collect::<Vec<_>>();
    counts.sort_by_key(|count| horizon_order(&count.horizon));
    counts
}

pub(super) fn unique_sorted<'a>(values: impl Iterator<Item = &'a str>) -> Vec<String> {
    values
        .map(ToOwned::to_owned)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}
