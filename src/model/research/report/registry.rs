use serde::{Deserialize, Serialize};

use crate::model::{ResearchAggregateRegistryStage, ResearchBias, SurvivalBand};

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
