use serde::{Deserialize, Serialize};

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
