use std::collections::BTreeMap;

#[derive(Default)]
pub(super) struct AllocationCounts {
    symbol_counts: BTreeMap<String, usize>,
    family_counts: BTreeMap<String, usize>,
}

impl AllocationCounts {
    pub(super) fn record(&mut self, symbol: &str, family: &str) {
        increment_count(&mut self.symbol_counts, symbol);
        increment_count(&mut self.family_counts, family);
    }

    pub(super) fn symbol(&self, symbol: &str) -> usize {
        self.symbol_counts.get(symbol).copied().unwrap_or(0)
    }

    pub(super) fn family(&self, family: &str) -> usize {
        self.family_counts.get(family).copied().unwrap_or(0)
    }
}

fn increment_count(counts: &mut BTreeMap<String, usize>, key: &str) {
    counts
        .entry(key.to_owned())
        .and_modify(|count| *count += 1)
        .or_insert(1);
}
