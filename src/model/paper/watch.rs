use super::super::{ResearchBias, SurvivalBand};
use super::risk::{PaperExpectedCostProfile, PaperExpectedRiskProfile};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PaperWatchCandidate {
    pub paper_watch_candidate_id: String,
    pub candidate_id: String,
    pub candidate_lifecycle_key: String,
    pub symbol_canonical: String,
    pub source_research_run_id: String,
    pub source_research_packet_id: String,
    pub source_research_bias: ResearchBias,
    pub historical_survival_band: SurvivalBand,
    pub admission_reason_codes: Vec<String>,
    pub blocked_promotion_reason_codes: Vec<String>,
    pub replay_sample_summary: PaperWatchReplaySampleSummary,
    pub expected_cost_profile: PaperExpectedCostProfile,
    pub expected_risk_profile: PaperExpectedRiskProfile,
    pub target_max_holding_hours: u32,
    pub absolute_max_holding_hours: u32,
    pub force_flat_policy: String,
    pub paper_start_recommendation: String,
    pub safety: PaperWatchSafety,
    pub created_at_ms: i64,
    pub schema_version: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PaperWatchReplaySampleSummary {
    pub research_aggregate_key: String,
    pub replay_run_count: usize,
    pub completed_count: usize,
    pub positive_net_count: usize,
    pub non_positive_net_count: usize,
    pub missing_market_replay_data_count: usize,
    pub insufficient_evidence_count: usize,
    pub effective_completed_sample_weight: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weighted_mean_net_after_cost_bps: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weighted_profit_factor_ppm: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PaperWatchSafety {
    pub paper_only: bool,
    pub live_enabled: bool,
    pub order_execution_enabled: bool,
    pub execution_approval_emitted: bool,
}
