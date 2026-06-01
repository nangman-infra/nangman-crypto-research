use serde::{Deserialize, Serialize};

use crate::model::{RegimeReplaySummary, ResearchBias, SurvivalBand, TrainValidationSplitSummary};

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
