use super::super::SurvivalBand;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PaperExpectedCostProfile {
    pub fee_model_version: String,
    pub slippage_model_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_cost_bps: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_stressed_mean_net_after_cost_bps: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PaperExpectedRiskProfile {
    pub survival_band: SurvivalBand,
    pub max_drawdown_band: String,
    pub positive_net_count: usize,
    pub non_positive_net_count: usize,
}
