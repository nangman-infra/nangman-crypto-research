pub(super) struct NextActionInput<'a> {
    pub(super) horizon_ms: Option<i64>,
    pub(super) horizon_due_ms: Option<i64>,
    pub(super) latest_l1_as_of_ms: Option<i64>,
    pub(super) matched_count: usize,
    pub(super) reason_codes: &'a [String],
    pub(super) completed_count: usize,
    pub(super) min_completed: usize,
    pub(super) inferred_unseen_window_count: usize,
    pub(super) required_unseen_window_count: usize,
    pub(super) train_validation_split_required: bool,
    pub(super) train_validation_split_materialized: bool,
    pub(super) liquidity_filter_required: bool,
    pub(super) liquidity_filter_materialized_count: usize,
}

pub(super) fn next_action(input: NextActionInput<'_>) -> String {
    if input.horizon_ms.is_none() {
        return "define_horizon_duration".to_owned();
    }
    let Some(horizon_due_ms) = input.horizon_due_ms else {
        return "define_replay_boundary".to_owned();
    };
    let Some(latest_l1_as_of_ms) = input.latest_l1_as_of_ms else {
        return "discover_latest_market_l1_as_of".to_owned();
    };
    if latest_l1_as_of_ms < horizon_due_ms {
        return "wait_for_market_l1_horizon".to_owned();
    }
    if input.matched_count == 0 {
        return "run_research_replay_for_horizon".to_owned();
    }
    if contains_reason(input.reason_codes, "missing_native_replay_market_data")
        || contains_reason(input.reason_codes, "native_replay_horizon_not_materialized")
    {
        return "extend_market_l1_horizon_coverage".to_owned();
    }
    if input.completed_count == 0 {
        return "materialize_completed_native_replay_sample".to_owned();
    }
    if input.completed_count < input.min_completed {
        return "accumulate_completed_native_replay_samples".to_owned();
    }
    if input.inferred_unseen_window_count < input.required_unseen_window_count {
        return "materialize_unseen_replay_windows".to_owned();
    }
    if input.train_validation_split_required && !input.train_validation_split_materialized {
        return "materialize_train_validation_split".to_owned();
    }
    if input.liquidity_filter_required
        && input.liquidity_filter_materialized_count < input.completed_count
    {
        return "materialize_liquidity_filter_inputs".to_owned();
    }
    if !input.reason_codes.is_empty() {
        return "inspect_remaining_gate_reasons".to_owned();
    }
    "promotion_gate_ready_for_review".to_owned()
}

fn contains_reason(reason_codes: &[String], expected: &str) -> bool {
    reason_codes.iter().any(|reason| reason == expected)
}
