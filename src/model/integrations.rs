use serde::{Deserialize, Serialize};

use super::{ResearchBias, SurvivalBand};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HypothesisOutput {
    None,
    L1SummaryOnly,
    L2HypothesisAttached,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SummaryFinding {
    pub candidate_id: String,
    pub candidate_lifecycle_key: String,
    pub bias: ResearchBias,
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct OssAdapterRun {
    pub oss_adapter_run_id: String,
    pub adapter_name: String,
    pub adapter_version: String,
    pub candidate_lifecycle_key: String,
    #[serde(default)]
    pub input_artifact_refs: Vec<String>,
    pub market_window: String,
    pub fee_model_used: String,
    pub slippage_model_used: String,
    pub trade_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub net_return_bps: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_drawdown_bps: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profit_factor: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sharpe_like_score: Option<f64>,
    pub lookahead_check_result: String,
    pub holding_horizon_check_result: String,
    #[serde(default)]
    pub adapter_warnings: Vec<String>,
    pub normalized_verdict_bias: OssAdapterVerdictBias,
    pub schema_version: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OssAdapterVerdictBias {
    PruneBias,
    RetestBias,
    PromoteToReplayBias,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PortfolioAllocationSnapshot {
    pub portfolio_allocation_snapshot_id: String,
    pub schema_version: String,
    pub allocation_policy_version: String,
    pub computed_at_ms: i64,
    pub market_regime: String,
    pub active_candidate_count: usize,
    pub max_total_notional_pct: f64,
    pub max_symbol_notional_pct: f64,
    pub max_candidate_notional_pct: f64,
    pub max_regime_notional_pct: f64,
    pub candidate_allocations: Vec<CandidateAllocation>,
    pub rejected_candidates: Vec<PortfolioRiskRejectEvent>,
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CandidateAllocation {
    pub candidate_lifecycle_key: String,
    pub symbol_canonical: String,
    pub strategy_id: String,
    pub allocation_weight: f64,
    pub max_notional_pct: f64,
    pub correlation_bucket: String,
    pub holding_deadline_ms: i64,
    pub paper_survival_band: SurvivalBand,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PortfolioRiskRejectEvent {
    pub portfolio_risk_reject_event_id: String,
    pub schema_version: String,
    pub candidate_lifecycle_key: String,
    pub symbol_canonical: String,
    pub reason: String,
    pub computed_at_ms: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PortfolioReduceOnlySignal {
    pub portfolio_reduce_only_signal_id: String,
    pub schema_version: String,
    pub symbol_canonical: String,
    pub reason: String,
    pub computed_at_ms: i64,
}
