use serde::{Deserialize, Serialize};

use super::replay::{RegimeReplaySummary, TrainValidationSplitSummary};
use super::status::{
    ResearchAggregateRegistryStage, ResearchBias, ResearchRunStatus, SurvivalBand,
};
use crate::model::{
    HypothesisOutput, PortfolioAllocationSnapshot, PortfolioReduceOnlySignal,
    PortfolioRiskRejectEvent, ShadowValidationRun, SummaryFinding,
};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ResearchRunReport {
    pub research_run_report_id: String,
    pub research_packet_id: String,
    pub source_candidate_ids: Vec<String>,
    pub run_scope: String,
    pub partition_count: usize,
    pub top_symbols: Vec<String>,
    pub top_families: Vec<String>,
    pub surviving_candidate_keys: Vec<String>,
    pub pruned_candidate_keys: Vec<String>,
    pub retest_candidate_keys: Vec<String>,
    pub shadow_validation_runs: Vec<ShadowValidationRun>,
    #[serde(default)]
    pub paper_watch_candidates: Vec<String>,
    pub paper_trade_candidates: Vec<String>,
    pub oss_adapter_run_ids: Vec<String>,
    pub oss_adapter_reject_count: usize,
    pub portfolio_allocation_snapshot: Option<PortfolioAllocationSnapshot>,
    pub portfolio_risk_reject_events: Vec<PortfolioRiskRejectEvent>,
    pub portfolio_reduce_only_signals: Vec<PortfolioReduceOnlySignal>,
    pub hypothesis_outputs: HypothesisOutput,
    pub research_gate_policy: ResearchGatePolicy,
    pub partition_aggregates: Vec<ResearchPartitionAggregate>,
    pub summary_findings: Vec<SummaryFinding>,
    pub research_run_status: ResearchRunStatus,
    pub created_at_ms: i64,
    pub replay_run_ids: Vec<String>,
    pub invalid_input_candidate_keys: Vec<String>,
    pub schema_version: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ResearchGatePolicy {
    pub policy_version: String,
    pub min_completed_samples_for_shadow: usize,
    pub min_win_rate_ppm_for_shadow: u64,
    pub min_profit_factor_ppm_for_shadow: u64,
    pub min_mean_net_after_cost_bps_for_shadow: f64,
    pub max_missing_or_insufficient_ratio_ppm_for_shadow: u64,
    pub min_market_regime_label_count_for_shadow: usize,
    pub cost_stress_multiplier_for_shadow: f64,
    pub full_weight_sample_max_age_days: u64,
    pub decayed_sample_max_age_days: u64,
    pub expired_sample_max_age_days: u64,
    pub decayed_sample_weight: f64,
    pub stale_sample_weight: f64,
    pub allow_promote_to_paper_bias: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ResearchPartitionAggregate {
    pub research_aggregate_key: String,
    pub research_partition_keys: Vec<String>,
    pub source_candidate_ids: Vec<String>,
    pub source_candidate_lifecycle_keys: Vec<String>,
    pub symbol_canonical: String,
    pub hypothesis_type: String,
    pub validation_adapter: String,
    pub strategy_id_or_family: String,
    pub parameter_variant_id: String,
    pub replay_run_count: usize,
    pub active_replay_run_count: usize,
    pub expired_replay_run_count: usize,
    pub completed_count: usize,
    pub decayed_completed_count: usize,
    pub expired_completed_count: usize,
    pub effective_completed_sample_weight: f64,
    pub invalid_input_count: usize,
    pub missing_market_replay_data_count: usize,
    pub insufficient_evidence_count: usize,
    pub liquidity_filter_materialized_count: usize,
    pub liquidity_filter_passed_count: usize,
    pub liquidity_filter_failed_count: usize,
    pub positive_net_count: usize,
    pub non_positive_net_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub win_rate_ppm: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mean_raw_return_bps: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mean_btc_adjusted_return_bps: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mean_net_after_cost_bps: Option<f64>,
    pub gross_positive_net_bps: f64,
    pub gross_negative_net_bps_abs: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profit_factor_ppm: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weighted_win_rate_ppm: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weighted_mean_net_after_cost_bps: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weighted_profit_factor_ppm: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_cost_bps: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_stressed_mean_net_after_cost_bps: Option<f64>,
    pub distinct_replay_window_count: usize,
    pub inferred_unseen_window_count: usize,
    pub market_regime_labels: Vec<String>,
    pub regime_summaries: Vec<RegimeReplaySummary>,
    pub train_validation_split_summary: TrainValidationSplitSummary,
    pub survival_band: SurvivalBand,
    pub gate_bias: ResearchBias,
    pub gate_reason_codes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ResearchAggregateRegistryRecord {
    pub research_aggregate_registry_record_id: String,
    pub research_run_report_id: String,
    pub research_packet_id: String,
    pub run_scope: String,
    pub research_aggregate_key: String,
    pub source_candidate_ids: Vec<String>,
    pub source_candidate_lifecycle_keys: Vec<String>,
    pub symbol_canonical: String,
    pub hypothesis_type: String,
    pub validation_adapter: String,
    pub strategy_id_or_family: String,
    pub parameter_variant_id: String,
    pub current_research_stage: ResearchAggregateRegistryStage,
    pub gate_bias: ResearchBias,
    pub survival_band: SurvivalBand,
    pub replay_run_count: usize,
    pub active_replay_run_count: usize,
    pub expired_replay_run_count: usize,
    pub completed_count: usize,
    pub effective_completed_sample_weight: f64,
    pub positive_net_count: usize,
    pub non_positive_net_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weighted_win_rate_ppm: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weighted_mean_net_after_cost_bps: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weighted_profit_factor_ppm: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_stressed_mean_net_after_cost_bps: Option<f64>,
    pub market_regime_labels: Vec<String>,
    pub latest_reason_codes: Vec<String>,
    pub linked_shadow_validation_run_ids: Vec<String>,
    pub created_at_ms: i64,
    pub schema_version: String,
}
