use super::super::{ShadowValidationStatus, SurvivalBand};
use super::risk::{PaperExpectedCostProfile, PaperExpectedRiskProfile};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PaperTradeCandidate {
    pub paper_trade_candidate_id: String,
    pub candidate_lifecycle_key: String,
    pub symbol_canonical: String,
    pub source_research_run_id: String,
    pub historical_survival_band: SurvivalBand,
    pub shadow_summary: PaperShadowSummary,
    pub expected_cost_profile: PaperExpectedCostProfile,
    pub expected_risk_profile: PaperExpectedRiskProfile,
    pub target_max_holding_hours: u32,
    pub absolute_max_holding_hours: u32,
    pub force_flat_policy: String,
    pub paper_start_recommendation: String,
    pub schema_version: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PaperShadowSummary {
    pub shadow_validation_run_id: String,
    pub status: ShadowValidationStatus,
    pub passed: bool,
    pub completed_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mean_net_after_cost_bps: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub win_rate_ppm: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profit_factor_ppm: Option<u64>,
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PaperTradeRun {
    pub paper_trade_run_id: String,
    pub candidate_lifecycle_key: String,
    pub symbol_canonical: String,
    pub source_research_run_id: String,
    pub paper_account_profile_id: String,
    pub started_at_ms: i64,
    pub ended_at_ms: i64,
    pub entry_count: usize,
    pub exit_count: usize,
    pub max_drawdown_band: String,
    pub net_result_band: String,
    pub survival_result: String,
    pub schema_version: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PaperTradeSummary {
    pub paper_trade_summary_id: String,
    pub paper_trade_run_id: String,
    pub candidate_lifecycle_key: String,
    pub summary_window: String,
    pub entry_behavior_summary: String,
    pub exit_behavior_summary: String,
    pub cost_behavior_summary: String,
    pub risk_behavior_summary: String,
    pub promote_recommendation: String,
    pub schema_version: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PaperTradeMark {
    pub paper_trade_mark_id: String,
    pub paper_trade_run_id: String,
    pub candidate_lifecycle_key: String,
    pub symbol_canonical: String,
    pub marked_at_ms: i64,
    pub mark_source: String,
    pub net_result_band: String,
    pub survival_result: String,
    pub schema_version: String,
}
