use std::collections::{BTreeMap, BTreeSet};

pub(super) fn unique_join<'a>(values: impl Iterator<Item = &'a str>) -> String {
    let values = values
        .filter_map(|value| {
            let trimmed = value.trim();
            (!trimmed.is_empty()).then(|| trimmed.to_owned())
        })
        .collect::<BTreeSet<_>>();
    if values.is_empty() {
        "없음".to_owned()
    } else {
        values.into_iter().collect::<Vec<_>>().join(", ")
    }
}

pub(super) fn count_join<'a>(values: impl Iterator<Item = &'a str>) -> String {
    let mut counts = BTreeMap::<String, usize>::new();
    for value in values {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            *counts.entry(trimmed.to_owned()).or_default() += 1;
        }
    }
    if counts.is_empty() {
        return "없음".to_owned();
    }
    counts
        .into_iter()
        .map(|(value, count)| format!("{value} {count}개"))
        .collect::<Vec<_>>()
        .join(", ")
}
