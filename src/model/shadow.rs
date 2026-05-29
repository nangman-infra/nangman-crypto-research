use serde::{Deserialize, Serialize};

use super::{
    ABSOLUTE_MAX_HOLDING_HOURS, HOLDING_POLICY_VERSION, PAPER_TRADE_CANDIDATE_SCHEMA_VERSION,
    SurvivalBand, TARGET_MAX_HOLDING_HOURS,
};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ShadowValidationRun {
    pub shadow_validation_run_id: String,
    pub candidate_lifecycle_key: String,
    pub symbol_canonical: String,
    pub trigger_research_run_id: String,
    pub start_condition_summary: ShadowStartConditionSummary,
    pub expected_survival_band: SurvivalBand,
    pub watch_window_policy: ShadowWatchWindowPolicy,
    pub termination_policy: ShadowTerminationPolicy,
    pub holding_policy: HoldingPolicy,
    #[serde(default)]
    pub status: ShadowValidationStatus,
    #[serde(default)]
    pub passed: bool,
    #[serde(default = "default_paper_trade_candidate_contract_version")]
    pub paper_trade_candidate_contract_version: String,
    pub schema_version: String,
}

fn default_paper_trade_candidate_contract_version() -> String {
    PAPER_TRADE_CANDIDATE_SCHEMA_VERSION.to_owned()
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ShadowValidationStatus {
    #[default]
    Pending,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct HoldingPolicy {
    pub target_max_holding_hours: u32,
    pub absolute_max_holding_hours: u32,
    pub absolute_exit_deadline_ms: i64,
    pub force_flat_policy: String,
    pub overnight_risk_exception: bool,
    pub holding_policy_version: String,
}

impl Default for HoldingPolicy {
    fn default() -> Self {
        Self {
            target_max_holding_hours: TARGET_MAX_HOLDING_HOURS,
            absolute_max_holding_hours: ABSOLUTE_MAX_HOLDING_HOURS,
            absolute_exit_deadline_ms: 0,
            force_flat_policy: "daily_or_ttl_exit".to_owned(),
            overnight_risk_exception: false,
            holding_policy_version: HOLDING_POLICY_VERSION.to_owned(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ShadowStartConditionSummary {
    pub research_aggregate_key: String,
    pub gate_policy_version: String,
    pub completed_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mean_net_after_cost_bps: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub win_rate_ppm: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profit_factor_ppm: Option<u64>,
    pub gate_reason_codes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ShadowWatchWindowPolicy {
    pub mode: String,
    pub min_shadow_samples: usize,
    pub max_shadow_age_days: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ShadowTerminationPolicy {
    pub prune_on_non_positive_mean_net: bool,
    pub prune_on_max_age_without_samples: bool,
    pub no_order_execution: bool,
}
