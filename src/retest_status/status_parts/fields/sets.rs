use std::collections::BTreeSet;

pub(in crate::retest_status) fn unique_sorted_strings<'a>(
    values: impl Iterator<Item = &'a str>,
) -> Vec<String> {
    values
        .map(ToOwned::to_owned)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(in crate::retest_status::status_parts) fn intersection_sorted(
    left: &BTreeSet<String>,
    right: &BTreeSet<String>,
) -> Vec<String> {
    left.intersection(right).cloned().collect()
}

pub(in crate::retest_status::status_parts) fn difference_sorted(
    left: &BTreeSet<String>,
    right: &BTreeSet<String>,
) -> Vec<String> {
    left.difference(right).cloned().collect()
}
