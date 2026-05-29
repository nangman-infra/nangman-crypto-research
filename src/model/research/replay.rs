use serde::{Deserialize, Serialize};

use super::status::{LiquidityFilterStatus, ReplayRunStatus, ResearchBias};
use crate::model::{
    DataQualitySummaryRef, HoldingPolicy, SourceIndependenceSummary, SymbolResolutionTrace,
};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ReplayRun {
    pub replay_run_id: String,
    pub source_candidate_id: String,
    pub source_candidate_lifecycle_key: String,
    pub research_partition_key: String,
    pub research_aggregate_key: String,
    pub symbol_canonical: String,
    pub decision_available_at_ms: i64,
    pub symbol_universe_snapshot_id: String,
    pub universe_as_of_ms: i64,
    pub approved_universe_symbol: bool,
    pub hypothesis_type: String,
    pub validation_adapter: String,
    pub strategy_id_or_family: String,
    pub window_start_ms: i64,
    pub window_end_ms: i64,
    pub forbidden_lookahead_boundary_ms: i64,
    #[serde(default)]
    pub holding_policy: HoldingPolicy,
    pub data_quality_summary_ref: DataQualitySummaryRef,
    pub source_independence_summary: SourceIndependenceSummary,
    pub symbol_resolution_trace_ref: Vec<SymbolResolutionTrace>,
    pub parameter_variant_id: String,
    pub cost_model_version: String,
    pub validation_recipe_version: String,
    pub result_summary: ReplayResultSummary,
    pub schema_version: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ReplayRunIndexRecord {
    pub replay_run_index_record_id: String,
    pub research_run_report_id: String,
    pub research_packet_id: String,
    pub run_scope: String,
    pub replay_run_id: String,
    pub replay_run_uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replay_run_s3_bucket: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replay_run_s3_key: Option<String>,
    pub source_candidate_id: String,
    pub source_candidate_lifecycle_key: String,
    pub research_partition_key: String,
    pub research_aggregate_key: String,
    pub symbol_canonical: String,
    pub decision_available_at_ms: i64,
    pub hypothesis_type: String,
    pub validation_adapter: String,
    pub strategy_id_or_family: String,
    pub parameter_variant_id: String,
    pub window_start_ms: i64,
    pub window_end_ms: i64,
    pub created_at_ms: i64,
    pub schema_version: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ReplayResultSummary {
    pub status: ReplayRunStatus,
    pub bias: ResearchBias,
    pub reason_codes: Vec<String>,
    pub matched_market_delta_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_return_bps: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub btc_adjusted_return_bps: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub net_after_cost_bps: Option<f64>,
    pub estimated_cost_bps: f64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub market_regime_labels: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub liquidity_filter_summary: Option<LiquidityFilterSummary>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct LiquidityFilterSummary {
    pub status: LiquidityFilterStatus,
    pub reason_codes: Vec<String>,
    pub observed_metric_count: usize,
    pub positive_volume_metric_count: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct RegimeReplaySummary {
    pub regime_label: String,
    pub completed_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mean_net_after_cost_bps: Option<f64>,
    pub positive_net_count: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct TrainValidationSplitSummary {
    pub required: bool,
    pub materialized: bool,
    pub train_completed_count: usize,
    pub validation_completed_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub train_mean_net_after_cost_bps: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_mean_net_after_cost_bps: Option<f64>,
    pub train_positive_net_count: usize,
    pub validation_positive_net_count: usize,
    pub passed: bool,
}
