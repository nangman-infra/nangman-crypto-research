pub const DEFAULT_FOCUSED_RETEST_ACTIONS: &[&str] = &[
    "run_research_replay_for_horizon",
    "accumulate_completed_native_replay_samples",
    "materialize_completed_native_replay_sample",
];

pub fn default_focused_retest_actions() -> Vec<String> {
    DEFAULT_FOCUSED_RETEST_ACTIONS
        .iter()
        .map(|action| (*action).to_owned())
        .collect()
}

pub fn parse_focused_retest_actions(raw: &str) -> Vec<String> {
    let mut actions = raw
        .split(',')
        .map(str::trim)
        .filter(|action| !action.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    actions.sort();
    actions.dedup();
    actions
}
