use crate::model::{DEFAULT_RESEARCH_GATE_POLICY_VERSION, ResearchGatePolicy};

pub fn default_research_gate_policy() -> ResearchGatePolicy {
    ResearchGatePolicy {
        policy_version: DEFAULT_RESEARCH_GATE_POLICY_VERSION.to_owned(),
        min_completed_samples_for_shadow: 30,
        min_win_rate_ppm_for_shadow: 500_000,
        min_profit_factor_ppm_for_shadow: 1_300_000,
        min_mean_net_after_cost_bps_for_shadow: 5.0,
        max_missing_or_insufficient_ratio_ppm_for_shadow: 200_000,
        min_market_regime_label_count_for_shadow: 1,
        cost_stress_multiplier_for_shadow: 2.0,
        full_weight_sample_max_age_days: 30,
        decayed_sample_max_age_days: 60,
        expired_sample_max_age_days: 90,
        decayed_sample_weight: 0.7,
        stale_sample_weight: 0.4,
        allow_promote_to_paper_bias: false,
    }
}
