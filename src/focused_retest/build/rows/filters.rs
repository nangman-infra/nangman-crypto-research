use std::collections::BTreeSet;

pub(super) struct FocusRowFilters<'a> {
    action_set: BTreeSet<&'a str>,
    lifecycle_filter: BTreeSet<&'a str>,
}

impl<'a> FocusRowFilters<'a> {
    pub(super) fn new(actions: &'a [String], candidate_lifecycle_key_filter: &'a [String]) -> Self {
        Self {
            action_set: actions.iter().map(String::as_str).collect(),
            lifecycle_filter: candidate_lifecycle_key_filter
                .iter()
                .map(String::as_str)
                .collect(),
        }
    }

    pub(super) fn allows_lifecycle_key(&self, candidate_lifecycle_key: Option<&str>) -> bool {
        self.lifecycle_filter.is_empty()
            || candidate_lifecycle_key.is_some_and(|key| self.lifecycle_filter.contains(key))
    }

    pub(super) fn allows_action(&self, next_action: &str) -> bool {
        self.action_set.contains(next_action)
    }
}
