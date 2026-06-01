use super::GateEvaluationInputs;

pub(super) fn promotion_blockers(
    inputs: &GateEvaluationInputs<'_>,
    mean_net_after_cost_bps: f64,
) -> Vec<String> {
    let mut blockers = vec!["native_replay_positive_but_promotion_blocked".to_owned()];

    add_sample_blockers(&mut blockers, inputs);
    add_performance_blockers(&mut blockers, inputs, mean_net_after_cost_bps);
    add_validation_blockers(&mut blockers, inputs);
    add_liquidity_blockers(&mut blockers, inputs);
    add_regime_blockers(&mut blockers, inputs);
    add_survival_blockers(&mut blockers, inputs);

    blockers
}

fn add_sample_blockers(blockers: &mut Vec<String>, inputs: &GateEvaluationInputs<'_>) {
    if inputs.accumulator.completed_count < inputs.policy.min_completed_samples_for_shadow {
        blockers.push("promotion_sample_count_below_minimum".to_owned());
    }
    if inputs.accumulator.effective_completed_sample_weight
        < inputs.policy.min_completed_samples_for_shadow as f64
    {
        blockers.push("promotion_effective_sample_weight_below_minimum".to_owned());
    }
}

fn add_performance_blockers(
    blockers: &mut Vec<String>,
    inputs: &GateEvaluationInputs<'_>,
    mean_net_after_cost_bps: f64,
) {
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
}

fn add_validation_blockers(blockers: &mut Vec<String>, inputs: &GateEvaluationInputs<'_>) {
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
}

fn add_liquidity_blockers(blockers: &mut Vec<String>, inputs: &GateEvaluationInputs<'_>) {
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
}

fn add_regime_blockers(blockers: &mut Vec<String>, inputs: &GateEvaluationInputs<'_>) {
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
}

fn add_survival_blockers(blockers: &mut Vec<String>, inputs: &GateEvaluationInputs<'_>) {
    if inputs
        .cost_stressed_mean_net_after_cost_bps
        .is_none_or(|value| value <= 0.0)
    {
        blockers.push("cost_stress_survival_not_proven".to_owned());
    }
}

#[cfg(test)]
mod tests {
    use super::promotion_blockers;
    use crate::gate::aggregate::accumulator::AggregateAccumulator;
    use crate::gate::aggregate::evaluation::GateEvaluationInputs;
    use crate::gate::policy::default_research_gate_policy;
    use crate::model::{RegimeReplaySummary, ResearchGatePolicy, TrainValidationSplitSummary};
    use std::collections::BTreeSet;

    #[test]
    fn promotion_blockers_accepts_full_promotion_evidence() {
        let mut accumulator = promotion_ready_accumulator();
        accumulator.required_unseen_windows = 1;
        let policy = default_research_gate_policy();
        let split = train_validation_split(false, false, false);
        let regimes = vec![positive_regime_summary()];
        let inputs = gate_inputs(&accumulator, &policy, &split, &regimes);

        assert_eq!(
            promotion_blockers(&inputs, 6.0),
            vec!["native_replay_positive_but_promotion_blocked".to_owned()]
        );
    }

    #[test]
    fn promotion_blockers_reports_validation_and_liquidity_gaps() {
        let mut accumulator = promotion_ready_accumulator();
        accumulator.required_unseen_windows = 2;
        accumulator.train_validation_split_required = true;
        accumulator.liquidity_filter_required = true;
        accumulator.liquidity_filter_materialized_count = 1;
        accumulator.liquidity_filter_failed_count = 1;
        let policy = default_research_gate_policy();
        let split = train_validation_split(true, false, false);
        let regimes = vec![positive_regime_summary()];
        let inputs = GateEvaluationInputs {
            inferred_unseen_window_count: 1,
            ..gate_inputs(&accumulator, &policy, &split, &regimes)
        };

        let blockers = promotion_blockers(&inputs, 6.0);

        assert!(blockers.contains(&"unseen_window_validation_not_proven".to_owned()));
        assert!(blockers.contains(&"train_validation_split_not_materialized".to_owned()));
        assert!(blockers.contains(&"liquidity_filter_not_materialized".to_owned()));
        assert!(blockers.contains(&"liquidity_filter_failed".to_owned()));
    }

    #[test]
    fn promotion_blockers_reports_failed_split_after_materialization() {
        let mut accumulator = promotion_ready_accumulator();
        accumulator.train_validation_split_required = true;
        let policy = default_research_gate_policy();
        let split = train_validation_split(true, true, false);
        let regimes = vec![positive_regime_summary()];
        let inputs = gate_inputs(&accumulator, &policy, &split, &regimes);

        assert!(
            promotion_blockers(&inputs, 6.0).contains(&"train_validation_split_failed".to_owned())
        );
    }

    #[test]
    fn promotion_blockers_reports_sample_performance_and_survival_gaps() {
        let mut accumulator = promotion_ready_accumulator();
        accumulator.completed_count = 1;
        accumulator.effective_completed_sample_weight = 0.5;
        accumulator.market_regime_labels.clear();
        let policy = default_research_gate_policy();
        let split = train_validation_split(false, false, false);
        let regimes = vec![RegimeReplaySummary {
            regime_label: "medium_volatility".to_owned(),
            completed_count: 1,
            mean_net_after_cost_bps: None,
            positive_net_count: 0,
        }];
        let inputs = GateEvaluationInputs {
            win_rate_ppm: None,
            profit_factor_ppm: None,
            unavailable_ratio_ppm: policy.max_missing_or_insufficient_ratio_ppm_for_shadow + 1,
            cost_stressed_mean_net_after_cost_bps: None,
            ..gate_inputs(&accumulator, &policy, &split, &regimes)
        };

        let blockers = promotion_blockers(&inputs, 1.0);

        for expected in [
            "promotion_sample_count_below_minimum",
            "promotion_effective_sample_weight_below_minimum",
            "promotion_win_rate_below_minimum",
            "promotion_profit_factor_below_minimum",
            "promotion_mean_net_edge_below_minimum",
            "replay_data_unavailable_ratio_above_limit",
            "market_regime_context_missing",
            "regime_stability_not_proven",
            "cost_stress_survival_not_proven",
        ] {
            assert!(
                blockers.contains(&expected.to_owned()),
                "missing {expected}"
            );
        }
    }

    fn gate_inputs<'a>(
        accumulator: &'a AggregateAccumulator,
        policy: &'a ResearchGatePolicy,
        split: &'a TrainValidationSplitSummary,
        regimes: &'a [RegimeReplaySummary],
    ) -> GateEvaluationInputs<'a> {
        GateEvaluationInputs {
            accumulator,
            policy,
            win_rate_ppm: Some(policy.min_win_rate_ppm_for_shadow),
            mean_net_after_cost_bps: Some(policy.min_mean_net_after_cost_bps_for_shadow),
            profit_factor_ppm: Some(policy.min_profit_factor_ppm_for_shadow),
            unavailable_ratio_ppm: 0,
            inferred_unseen_window_count: accumulator.required_unseen_windows,
            cost_stressed_mean_net_after_cost_bps: Some(1.0),
            regime_summaries: regimes,
            train_validation_split_summary: split,
        }
    }

    fn promotion_ready_accumulator() -> AggregateAccumulator {
        let mut market_regime_labels = BTreeSet::new();
        market_regime_labels.insert("medium_volatility".to_owned());

        AggregateAccumulator {
            research_aggregate_key: "aggregate".to_owned(),
            research_partition_keys: BTreeSet::new(),
            source_candidate_ids: BTreeSet::new(),
            source_candidate_lifecycle_keys: BTreeSet::new(),
            symbol_canonical: "SUI".to_owned(),
            hypothesis_type: "risk_incident_watch".to_owned(),
            validation_adapter: "event_reaction_smoke".to_owned(),
            strategy_id_or_family: "strategy".to_owned(),
            parameter_variant_id: "base".to_owned(),
            replay_run_count: 30,
            active_replay_run_count: 30,
            expired_replay_run_count: 0,
            completed_count: 30,
            decayed_completed_count: 0,
            expired_completed_count: 0,
            effective_completed_sample_weight: 30.0,
            invalid_input_count: 0,
            missing_market_replay_data_count: 0,
            insufficient_evidence_count: 0,
            liquidity_filter_materialized_count: 30,
            liquidity_filter_passed_count: 30,
            liquidity_filter_failed_count: 0,
            positive_net_count: 30,
            non_positive_net_count: 0,
            raw_returns: Vec::new(),
            btc_adjusted_returns: Vec::new(),
            net_after_costs: Vec::new(),
            cost_estimates: Vec::new(),
            gross_positive_net_bps: 0.0,
            gross_negative_net_bps_abs: 0.0,
            weighted_positive_net_bps: 0.0,
            weighted_negative_net_bps_abs: 0.0,
            weighted_positive_sample_weight: 30.0,
            weighted_net_after_cost_sum: 180.0,
            active_replay_windows: BTreeSet::new(),
            market_regime_labels,
            completed_samples: Vec::new(),
            required_unseen_windows: 0,
            train_validation_split_required: false,
            liquidity_filter_required: false,
        }
    }

    fn positive_regime_summary() -> RegimeReplaySummary {
        RegimeReplaySummary {
            regime_label: "medium_volatility".to_owned(),
            completed_count: 30,
            mean_net_after_cost_bps: Some(6.0),
            positive_net_count: 30,
        }
    }

    fn train_validation_split(
        required: bool,
        materialized: bool,
        passed: bool,
    ) -> TrainValidationSplitSummary {
        TrainValidationSplitSummary {
            required,
            materialized,
            train_completed_count: 15,
            validation_completed_count: 15,
            train_mean_net_after_cost_bps: Some(6.0),
            validation_mean_net_after_cost_bps: Some(6.0),
            train_positive_net_count: 15,
            validation_positive_net_count: 15,
            passed,
        }
    }
}
