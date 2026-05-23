use crate::model::{
    DEFAULT_RESEARCH_GATE_POLICY_VERSION, IntelCandidateEvidenceBundle, LiquidityFilterStatus,
    RegimeReplaySummary, ReplayRun, ReplayRunStatus, ResearchBias, ResearchGatePolicy,
    ResearchPartitionAggregate, SurvivalBand, TrainValidationSplitSummary,
};
use std::collections::{BTreeMap, BTreeSet};

const PPM_DENOMINATOR: f64 = 1_000_000.0;
const MAX_REPORTED_PROFIT_FACTOR_PPM: u64 = 9_999_999_999;
const MS_PER_DAY: i64 = 24 * 60 * 60 * 1000;

pub fn default_research_gate_policy() -> ResearchGatePolicy {
    ResearchGatePolicy {
        policy_version: DEFAULT_RESEARCH_GATE_POLICY_VERSION.to_owned(),
        min_completed_samples_for_shadow: 30,
        min_win_rate_ppm_for_shadow: 500_000,
        min_profit_factor_ppm_for_shadow: 1_300_000,
        min_mean_net_after_cost_bps_for_shadow: 5.0,
        max_missing_or_insufficient_ratio_ppm_for_shadow: 200_000,
        min_market_regime_label_count_for_shadow: 1,
        cost_stress_multiplier_for_shadow: 2.0,
        full_weight_sample_max_age_days: 30,
        decayed_sample_max_age_days: 60,
        expired_sample_max_age_days: 90,
        decayed_sample_weight: 0.7,
        stale_sample_weight: 0.4,
        allow_promote_to_paper_bias: false,
    }
}

pub fn build_partition_aggregates(
    bundles: &[IntelCandidateEvidenceBundle],
    replay_runs: &[ReplayRun],
    policy: &ResearchGatePolicy,
    gate_as_of_ms: i64,
) -> Vec<ResearchPartitionAggregate> {
    let bundle_by_candidate_id = bundles
        .iter()
        .map(|bundle| (bundle.candidate_id.as_str(), bundle))
        .collect::<BTreeMap<_, _>>();
    let mut accumulators = BTreeMap::<String, AggregateAccumulator>::new();

    for run in replay_runs {
        let accumulator = accumulators
            .entry(run.research_aggregate_key.clone())
            .or_insert_with(|| AggregateAccumulator::new(run));
        accumulator.add_run(
            run,
            bundle_by_candidate_id
                .get(run.source_candidate_id.as_str())
                .copied(),
            policy,
            gate_as_of_ms,
        );
    }

    accumulators
        .into_values()
        .map(|accumulator| accumulator.finish(policy))
        .collect()
}

#[derive(Debug)]
struct AggregateAccumulator {
    research_aggregate_key: String,
    research_partition_keys: BTreeSet<String>,
    source_candidate_ids: BTreeSet<String>,
    source_candidate_lifecycle_keys: BTreeSet<String>,
    symbol_canonical: String,
    hypothesis_type: String,
    validation_adapter: String,
    strategy_id_or_family: String,
    parameter_variant_id: String,
    replay_run_count: usize,
    active_replay_run_count: usize,
    expired_replay_run_count: usize,
    completed_count: usize,
    decayed_completed_count: usize,
    expired_completed_count: usize,
    effective_completed_sample_weight: f64,
    invalid_input_count: usize,
    missing_market_replay_data_count: usize,
    insufficient_evidence_count: usize,
    liquidity_filter_materialized_count: usize,
    liquidity_filter_passed_count: usize,
    liquidity_filter_failed_count: usize,
    positive_net_count: usize,
    non_positive_net_count: usize,
    raw_returns: Vec<f64>,
    btc_adjusted_returns: Vec<f64>,
    net_after_costs: Vec<f64>,
    cost_estimates: Vec<f64>,
    gross_positive_net_bps: f64,
    gross_negative_net_bps_abs: f64,
    weighted_positive_net_bps: f64,
    weighted_negative_net_bps_abs: f64,
    weighted_positive_sample_weight: f64,
    weighted_net_after_cost_sum: f64,
    active_replay_windows: BTreeSet<String>,
    market_regime_labels: BTreeSet<String>,
    completed_samples: Vec<CompletedSample>,
    required_unseen_windows: usize,
    train_validation_split_required: bool,
    liquidity_filter_required: bool,
}

#[derive(Debug, Clone)]
struct CompletedSample {
    window_start_ms: i64,
    net_after_cost_bps: f64,
    estimated_cost_bps: f64,
    market_regime_labels: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DecayBand {
    Fresh,
    Decayed,
    Stale,
    Expired,
}

#[derive(Debug, Clone, Copy)]
struct ReplaySampleWeight {
    band: DecayBand,
    weight: f64,
}

impl AggregateAccumulator {
    fn new(run: &ReplayRun) -> Self {
        Self {
            research_aggregate_key: run.research_aggregate_key.clone(),
            research_partition_keys: BTreeSet::new(),
            source_candidate_ids: BTreeSet::new(),
            source_candidate_lifecycle_keys: BTreeSet::new(),
            symbol_canonical: run.symbol_canonical.clone(),
            hypothesis_type: run.hypothesis_type.clone(),
            validation_adapter: run.validation_adapter.clone(),
            strategy_id_or_family: run.strategy_id_or_family.clone(),
            parameter_variant_id: run.parameter_variant_id.clone(),
            replay_run_count: 0,
            active_replay_run_count: 0,
            expired_replay_run_count: 0,
            completed_count: 0,
            decayed_completed_count: 0,
            expired_completed_count: 0,
            effective_completed_sample_weight: 0.0,
            invalid_input_count: 0,
            missing_market_replay_data_count: 0,
            insufficient_evidence_count: 0,
            liquidity_filter_materialized_count: 0,
            liquidity_filter_passed_count: 0,
            liquidity_filter_failed_count: 0,
            positive_net_count: 0,
            non_positive_net_count: 0,
            raw_returns: Vec::new(),
            btc_adjusted_returns: Vec::new(),
            net_after_costs: Vec::new(),
            cost_estimates: Vec::new(),
            gross_positive_net_bps: 0.0,
            gross_negative_net_bps_abs: 0.0,
            weighted_positive_net_bps: 0.0,
            weighted_negative_net_bps_abs: 0.0,
            weighted_positive_sample_weight: 0.0,
            weighted_net_after_cost_sum: 0.0,
            active_replay_windows: BTreeSet::new(),
            market_regime_labels: BTreeSet::new(),
            completed_samples: Vec::new(),
            required_unseen_windows: 0,
            train_validation_split_required: false,
            liquidity_filter_required: false,
        }
    }

    fn add_run(
        &mut self,
        run: &ReplayRun,
        bundle: Option<&IntelCandidateEvidenceBundle>,
        policy: &ResearchGatePolicy,
        gate_as_of_ms: i64,
    ) {
        self.replay_run_count += 1;
        self.research_partition_keys
            .insert(run.research_partition_key.clone());
        self.source_candidate_ids
            .insert(run.source_candidate_id.clone());
        self.source_candidate_lifecycle_keys
            .insert(run.source_candidate_lifecycle_key.clone());
        let sample_weight = replay_sample_weight(gate_as_of_ms, run.window_end_ms, policy);
        if sample_weight.band == DecayBand::Expired {
            self.expired_replay_run_count += 1;
            if run.result_summary.status == ReplayRunStatus::Completed
                && run.result_summary.net_after_cost_bps.is_some()
            {
                self.expired_completed_count += 1;
            }
            return;
        }

        self.active_replay_run_count += 1;
        self.active_replay_windows
            .insert(format!("{}-{}", run.window_start_ms, run.window_end_ms));
        self.cost_estimates
            .push(run.result_summary.estimated_cost_bps);
        self.market_regime_labels
            .extend(run.result_summary.market_regime_labels.iter().cloned());

        if let Some(bundle) = bundle {
            self.required_unseen_windows = self
                .required_unseen_windows
                .max(bundle.validation_requirements.min_unseen_windows);
            self.train_validation_split_required |= bundle
                .validation_requirements
                .required_train_validation_split;
            self.liquidity_filter_required |=
                bundle.validation_requirements.include_liquidity_filter;
        }

        match run.result_summary.status {
            ReplayRunStatus::Completed => {
                self.completed_count += 1;
                if self.liquidity_filter_required {
                    self.add_liquidity_filter_summary(run);
                }
                if let Some(value) = run.result_summary.raw_return_bps {
                    self.raw_returns.push(value);
                }
                if let Some(value) = run.result_summary.btc_adjusted_return_bps {
                    self.btc_adjusted_returns.push(value);
                }
                if let Some(value) = run.result_summary.net_after_cost_bps {
                    self.net_after_costs.push(value);
                    self.effective_completed_sample_weight += sample_weight.weight;
                    if sample_weight.band != DecayBand::Fresh {
                        self.decayed_completed_count += 1;
                    }
                    self.weighted_net_after_cost_sum += value * sample_weight.weight;
                    self.completed_samples.push(CompletedSample {
                        window_start_ms: run.window_start_ms,
                        net_after_cost_bps: value,
                        estimated_cost_bps: run.result_summary.estimated_cost_bps,
                        market_regime_labels: run.result_summary.market_regime_labels.clone(),
                    });
                    if value > 0.0 {
                        self.positive_net_count += 1;
                        self.gross_positive_net_bps += value;
                        self.weighted_positive_net_bps += value * sample_weight.weight;
                        self.weighted_positive_sample_weight += sample_weight.weight;
                    } else {
                        self.non_positive_net_count += 1;
                        self.gross_negative_net_bps_abs += value.abs();
                        self.weighted_negative_net_bps_abs += value.abs() * sample_weight.weight;
                    }
                }
            }
            ReplayRunStatus::InvalidInput => {
                self.invalid_input_count += 1;
            }
            ReplayRunStatus::MissingMarketReplayData => {
                self.missing_market_replay_data_count += 1;
            }
            ReplayRunStatus::InsufficientEvidence => {
                self.insufficient_evidence_count += 1;
            }
        }
    }

    fn finish(self, policy: &ResearchGatePolicy) -> ResearchPartitionAggregate {
        let win_rate_ppm = ratio_ppm(self.positive_net_count, self.completed_count);
        let mean_raw_return_bps = mean(&self.raw_returns);
        let mean_btc_adjusted_return_bps = mean(&self.btc_adjusted_returns);
        let mean_net_after_cost_bps = mean(&self.net_after_costs);
        let unweighted_profit_factor_ppm =
            profit_factor_ppm(self.gross_positive_net_bps, self.gross_negative_net_bps_abs);
        let weighted_win_rate_ppm = weighted_ratio_ppm(
            self.weighted_positive_sample_weight,
            self.effective_completed_sample_weight,
        );
        let weighted_mean_net_after_cost_bps = weighted_mean(
            self.weighted_net_after_cost_sum,
            self.effective_completed_sample_weight,
        );
        let weighted_profit_factor_ppm = profit_factor_ppm(
            self.weighted_positive_net_bps,
            self.weighted_negative_net_bps_abs,
        );
        let estimated_cost_bps = mean(&self.cost_estimates);
        let distinct_replay_window_count = self.active_replay_windows.len();
        let inferred_unseen_window_count = distinct_replay_window_count.saturating_sub(1);
        let cost_stressed_mean_net_after_cost_bps = cost_stressed_mean_net_after_cost_bps(
            &self.completed_samples,
            policy.cost_stress_multiplier_for_shadow,
        );
        let regime_summaries = regime_summaries(&self.completed_samples);
        let train_validation_split_summary = train_validation_split_summary(
            self.train_validation_split_required,
            &self.completed_samples,
        );
        let unavailable_count = self.invalid_input_count
            + self.missing_market_replay_data_count
            + self.insufficient_evidence_count;
        let unavailable_ratio_ppm =
            ratio_ppm(unavailable_count, self.replay_run_count).unwrap_or_default();
        let gate_inputs = GateEvaluationInputs {
            accumulator: &self,
            policy,
            win_rate_ppm: weighted_win_rate_ppm,
            mean_net_after_cost_bps: weighted_mean_net_after_cost_bps,
            profit_factor_ppm: weighted_profit_factor_ppm,
            unavailable_ratio_ppm,
            inferred_unseen_window_count,
            cost_stressed_mean_net_after_cost_bps,
            regime_summaries: &regime_summaries,
            train_validation_split_summary: &train_validation_split_summary,
        };
        let (gate_bias, gate_reason_codes) = evaluate_gate(&gate_inputs);
        let survival_band = survival_band(
            &gate_bias,
            self.completed_count,
            mean_net_after_cost_bps,
            unweighted_profit_factor_ppm,
        );

        ResearchPartitionAggregate {
            research_aggregate_key: self.research_aggregate_key,
            research_partition_keys: self.research_partition_keys.into_iter().collect(),
            source_candidate_ids: self.source_candidate_ids.into_iter().collect(),
            source_candidate_lifecycle_keys: self
                .source_candidate_lifecycle_keys
                .into_iter()
                .collect(),
            symbol_canonical: self.symbol_canonical,
            hypothesis_type: self.hypothesis_type,
            validation_adapter: self.validation_adapter,
            strategy_id_or_family: self.strategy_id_or_family,
            parameter_variant_id: self.parameter_variant_id,
            replay_run_count: self.replay_run_count,
            active_replay_run_count: self.active_replay_run_count,
            expired_replay_run_count: self.expired_replay_run_count,
            completed_count: self.completed_count,
            decayed_completed_count: self.decayed_completed_count,
            expired_completed_count: self.expired_completed_count,
            effective_completed_sample_weight: self.effective_completed_sample_weight,
            invalid_input_count: self.invalid_input_count,
            missing_market_replay_data_count: self.missing_market_replay_data_count,
            insufficient_evidence_count: self.insufficient_evidence_count,
            liquidity_filter_materialized_count: self.liquidity_filter_materialized_count,
            liquidity_filter_passed_count: self.liquidity_filter_passed_count,
            liquidity_filter_failed_count: self.liquidity_filter_failed_count,
            positive_net_count: self.positive_net_count,
            non_positive_net_count: self.non_positive_net_count,
            win_rate_ppm,
            mean_raw_return_bps,
            mean_btc_adjusted_return_bps,
            mean_net_after_cost_bps,
            gross_positive_net_bps: self.gross_positive_net_bps,
            gross_negative_net_bps_abs: self.gross_negative_net_bps_abs,
            profit_factor_ppm: unweighted_profit_factor_ppm,
            weighted_win_rate_ppm,
            weighted_mean_net_after_cost_bps,
            weighted_profit_factor_ppm,
            estimated_cost_bps,
            cost_stressed_mean_net_after_cost_bps,
            distinct_replay_window_count,
            inferred_unseen_window_count,
            market_regime_labels: self.market_regime_labels.into_iter().collect(),
            regime_summaries,
            train_validation_split_summary,
            survival_band,
            gate_bias,
            gate_reason_codes,
        }
    }

    fn add_liquidity_filter_summary(&mut self, run: &ReplayRun) {
        let Some(summary) = run.result_summary.liquidity_filter_summary.as_ref() else {
            return;
        };
        match summary.status {
            LiquidityFilterStatus::Passed => {
                self.liquidity_filter_materialized_count += 1;
                self.liquidity_filter_passed_count += 1;
            }
            LiquidityFilterStatus::Failed => {
                self.liquidity_filter_materialized_count += 1;
                self.liquidity_filter_failed_count += 1;
            }
            LiquidityFilterStatus::NotMaterialized | LiquidityFilterStatus::NotRequired => {}
        }
    }
}

struct GateEvaluationInputs<'a> {
    accumulator: &'a AggregateAccumulator,
    policy: &'a ResearchGatePolicy,
    win_rate_ppm: Option<u64>,
    mean_net_after_cost_bps: Option<f64>,
    profit_factor_ppm: Option<u64>,
    unavailable_ratio_ppm: u64,
    inferred_unseen_window_count: usize,
    cost_stressed_mean_net_after_cost_bps: Option<f64>,
    regime_summaries: &'a [RegimeReplaySummary],
    train_validation_split_summary: &'a TrainValidationSplitSummary,
}

fn evaluate_gate(inputs: &GateEvaluationInputs<'_>) -> (ResearchBias, Vec<String>) {
    let Some(mean_net_after_cost_bps) = inputs.mean_net_after_cost_bps else {
        return (
            ResearchBias::RetestBias,
            vec!["no_completed_native_replay_samples".to_owned()],
        );
    };

    if mean_net_after_cost_bps <= 0.0 {
        return (
            ResearchBias::PruneBias,
            vec!["aggregate_net_edge_non_positive".to_owned()],
        );
    }

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

    if blockers.len() == 1 {
        return (
            ResearchBias::PromoteToShadowBias,
            vec!["deterministic_shadow_gate_passed".to_owned()],
        );
    }

    (ResearchBias::RetestBias, blockers)
}

fn survival_band(
    bias: &ResearchBias,
    completed_count: usize,
    mean_net_after_cost_bps: Option<f64>,
    profit_factor_ppm: Option<u64>,
) -> SurvivalBand {
    match bias {
        ResearchBias::PruneBias => SurvivalBand::Fragile,
        ResearchBias::PromoteToShadowBias => {
            if completed_count >= 100
                && mean_net_after_cost_bps.is_some_and(|value| value >= 20.0)
                && profit_factor_ppm.is_some_and(|value| value >= 2_000_000)
            {
                SurvivalBand::Exceptional
            } else {
                SurvivalBand::Stable
            }
        }
        ResearchBias::RetestBias => {
            if mean_net_after_cost_bps.is_some_and(|value| value > 0.0) {
                SurvivalBand::Conditional
            } else {
                SurvivalBand::Fragile
            }
        }
        ResearchBias::PromoteToPaperBias => SurvivalBand::Conditional,
    }
}

fn mean(values: &[f64]) -> Option<f64> {
    (!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64)
}

fn weighted_mean(weighted_sum: f64, weight_sum: f64) -> Option<f64> {
    (weight_sum > 0.0).then(|| weighted_sum / weight_sum)
}

fn train_validation_split_summary(
    required: bool,
    completed_samples: &[CompletedSample],
) -> TrainValidationSplitSummary {
    if !required {
        return TrainValidationSplitSummary {
            required,
            materialized: false,
            train_completed_count: 0,
            validation_completed_count: 0,
            train_mean_net_after_cost_bps: None,
            validation_mean_net_after_cost_bps: None,
            train_positive_net_count: 0,
            validation_positive_net_count: 0,
            passed: true,
        };
    }

    let mut samples = completed_samples.to_vec();
    samples.sort_by_key(|sample| sample.window_start_ms);
    let split_index = samples.len() / 2;
    let (train, validation) = samples.split_at(split_index);
    let train_nets = train
        .iter()
        .map(|sample| sample.net_after_cost_bps)
        .collect::<Vec<_>>();
    let validation_nets = validation
        .iter()
        .map(|sample| sample.net_after_cost_bps)
        .collect::<Vec<_>>();
    let train_mean_net_after_cost_bps = mean(&train_nets);
    let validation_mean_net_after_cost_bps = mean(&validation_nets);
    let materialized = !train.is_empty() && !validation.is_empty();
    let passed = materialized
        && train_mean_net_after_cost_bps.is_some_and(|value| value > 0.0)
        && validation_mean_net_after_cost_bps.is_some_and(|value| value > 0.0);

    TrainValidationSplitSummary {
        required,
        materialized,
        train_completed_count: train.len(),
        validation_completed_count: validation.len(),
        train_mean_net_after_cost_bps,
        validation_mean_net_after_cost_bps,
        train_positive_net_count: train
            .iter()
            .filter(|sample| sample.net_after_cost_bps > 0.0)
            .count(),
        validation_positive_net_count: validation
            .iter()
            .filter(|sample| sample.net_after_cost_bps > 0.0)
            .count(),
        passed,
    }
}

fn regime_summaries(completed_samples: &[CompletedSample]) -> Vec<RegimeReplaySummary> {
    let mut regime_nets = BTreeMap::<String, Vec<f64>>::new();
    for sample in completed_samples {
        for label in &sample.market_regime_labels {
            regime_nets
                .entry(label.clone())
                .or_default()
                .push(sample.net_after_cost_bps);
        }
    }

    regime_nets
        .into_iter()
        .map(|(regime_label, nets)| RegimeReplaySummary {
            regime_label,
            completed_count: nets.len(),
            mean_net_after_cost_bps: mean(&nets),
            positive_net_count: nets.iter().filter(|value| **value > 0.0).count(),
        })
        .collect()
}

fn cost_stressed_mean_net_after_cost_bps(
    completed_samples: &[CompletedSample],
    cost_stress_multiplier: f64,
) -> Option<f64> {
    if completed_samples.is_empty() {
        return None;
    }
    let extra_cost_multiplier = (cost_stress_multiplier - 1.0).max(0.0);
    let stressed = completed_samples
        .iter()
        .map(|sample| {
            sample.net_after_cost_bps - (sample.estimated_cost_bps * extra_cost_multiplier)
        })
        .collect::<Vec<_>>();
    mean(&stressed)
}

fn ratio_ppm(numerator: usize, denominator: usize) -> Option<u64> {
    if denominator == 0 {
        return None;
    }
    Some(((numerator as f64 / denominator as f64) * PPM_DENOMINATOR).round() as u64)
}

fn weighted_ratio_ppm(numerator_weight: f64, denominator_weight: f64) -> Option<u64> {
    if denominator_weight <= 0.0 {
        return None;
    }
    Some(((numerator_weight / denominator_weight) * PPM_DENOMINATOR).round() as u64)
}

fn profit_factor_ppm(gross_positive_net_bps: f64, gross_negative_net_bps_abs: f64) -> Option<u64> {
    if gross_negative_net_bps_abs > 0.0 {
        return Some(
            ((gross_positive_net_bps / gross_negative_net_bps_abs) * PPM_DENOMINATOR).round()
                as u64,
        );
    }
    (gross_positive_net_bps > 0.0).then_some(MAX_REPORTED_PROFIT_FACTOR_PPM)
}

fn replay_sample_weight(
    gate_as_of_ms: i64,
    window_end_ms: i64,
    policy: &ResearchGatePolicy,
) -> ReplaySampleWeight {
    let age_ms = gate_as_of_ms.saturating_sub(window_end_ms);
    let age_days = age_ms / MS_PER_DAY;

    if age_days > policy.expired_sample_max_age_days as i64 {
        return ReplaySampleWeight {
            band: DecayBand::Expired,
            weight: 0.0,
        };
    }
    if age_days > policy.decayed_sample_max_age_days as i64 {
        return ReplaySampleWeight {
            band: DecayBand::Stale,
            weight: policy.stale_sample_weight,
        };
    }
    if age_days > policy.full_weight_sample_max_age_days as i64 {
        return ReplaySampleWeight {
            band: DecayBand::Decayed,
            weight: policy.decayed_sample_weight,
        };
    }
    ReplaySampleWeight {
        band: DecayBand::Fresh,
        weight: 1.0,
    }
}
