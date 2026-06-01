use super::GateEvaluationInputs;

pub(super) fn promotion_blockers(
    inputs: &GateEvaluationInputs<'_>,
    mean_net_after_cost_bps: f64,
) -> Vec<String> {
    let mut blockers = vec!["native_replay_positive_but_promotion_blocked".to_owned()];

    if inputs.accumulator.completed_count < inputs.policy.min_completed_samples_for_shadow {
        blockers.push("promotion_sample_count_below_minimum".to_owned());
    }
    if inputs.accumulator.effective_completed_sample_weight
        < inputs.policy.min_completed_samples_for_shadow as f64
    {
        blockers.push("promotion_effective_sample_weight_below_minimum".to_owned());
    }
    if inputs
        .win_rate_ppm
        .is_none_or(|value| value < inputs.policy.min_win_rate_ppm_for_shadow)
    {
        blockers.push("promotion_win_rate_below_minimum".to_owned());
    }
    if inputs
        .profit_factor_ppm
        .is_none_or(|value| value < inputs.policy.min_profit_factor_ppm_for_shadow)
    {
        blockers.push("promotion_profit_factor_below_minimum".to_owned());
    }
    if mean_net_after_cost_bps < inputs.policy.min_mean_net_after_cost_bps_for_shadow {
        blockers.push("promotion_mean_net_edge_below_minimum".to_owned());
    }
    if inputs.unavailable_ratio_ppm
        > inputs
            .policy
            .max_missing_or_insufficient_ratio_ppm_for_shadow
    {
        blockers.push("replay_data_unavailable_ratio_above_limit".to_owned());
    }
    if inputs.inferred_unseen_window_count < inputs.accumulator.required_unseen_windows {
        blockers.push("unseen_window_validation_not_proven".to_owned());
    }
    if inputs.accumulator.train_validation_split_required
        && !inputs.train_validation_split_summary.materialized
    {
        blockers.push("train_validation_split_not_materialized".to_owned());
    }
    if inputs.accumulator.train_validation_split_required
        && inputs.train_validation_split_summary.materialized
        && !inputs.train_validation_split_summary.passed
    {
        blockers.push("train_validation_split_failed".to_owned());
    }
    if inputs.accumulator.liquidity_filter_required
        && inputs.accumulator.liquidity_filter_materialized_count
            < inputs.accumulator.completed_count
    {
        blockers.push("liquidity_filter_not_materialized".to_owned());
    }
    if inputs.accumulator.liquidity_filter_required
        && inputs.accumulator.liquidity_filter_failed_count > 0
    {
        blockers.push("liquidity_filter_failed".to_owned());
    }
    if inputs.accumulator.market_regime_labels.len()
        < inputs.policy.min_market_regime_label_count_for_shadow
    {
        blockers.push("market_regime_context_missing".to_owned());
    }
    if inputs.regime_summaries.iter().any(|summary| {
        summary
            .mean_net_after_cost_bps
            .is_none_or(|value| value <= 0.0)
    }) {
        blockers.push("regime_stability_not_proven".to_owned());
    }
    if inputs
        .cost_stressed_mean_net_after_cost_bps
        .is_none_or(|value| value <= 0.0)
    {
        blockers.push("cost_stress_survival_not_proven".to_owned());
    }

    blockers
}
