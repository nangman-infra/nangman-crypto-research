use super::CandidatePaperBuildInput;
use crate::model::{
    PAPER_TRADE_CANDIDATE_SCHEMA_VERSION, PaperExpectedCostProfile, PaperExpectedRiskProfile,
    PaperShadowSummary, PaperTradeCandidate,
};
use crate::paper::shared::max_drawdown_band;

pub(super) fn build_candidate(
    input: &CandidatePaperBuildInput<'_>,
    paper_candidate_id: &str,
) -> PaperTradeCandidate {
    PaperTradeCandidate {
        paper_trade_candidate_id: paper_candidate_id.to_owned(),
        candidate_lifecycle_key: input.candidate_lifecycle_key.to_owned(),
        symbol_canonical: input.aggregate.symbol_canonical.clone(),
        source_research_run_id: input.report.research_run_report_id.clone(),
        historical_survival_band: input.aggregate.survival_band.clone(),
        shadow_summary: PaperShadowSummary {
            shadow_validation_run_id: input.shadow_run.shadow_validation_run_id.clone(),
            status: input.shadow_run.status.clone(),
            passed: input.shadow_run.passed,
            completed_count: input.shadow_run.start_condition_summary.completed_count,
            mean_net_after_cost_bps: input
                .shadow_run
                .start_condition_summary
                .mean_net_after_cost_bps,
            win_rate_ppm: input.shadow_run.start_condition_summary.win_rate_ppm,
            profit_factor_ppm: input.shadow_run.start_condition_summary.profit_factor_ppm,
            reason_codes: input
                .shadow_run
                .start_condition_summary
                .gate_reason_codes
                .clone(),
        },
        expected_cost_profile: PaperExpectedCostProfile {
            fee_model_version: input.profile.fee_model_version.clone(),
            slippage_model_version: input.profile.slippage_model_version.clone(),
            estimated_cost_bps: input.aggregate.estimated_cost_bps,
            cost_stressed_mean_net_after_cost_bps: input
                .aggregate
                .cost_stressed_mean_net_after_cost_bps,
        },
        expected_risk_profile: PaperExpectedRiskProfile {
            survival_band: input.aggregate.survival_band.clone(),
            max_drawdown_band: max_drawdown_band(input.aggregate),
            positive_net_count: input.aggregate.positive_net_count,
            non_positive_net_count: input.aggregate.non_positive_net_count,
        },
        target_max_holding_hours: input.shadow_run.holding_policy.target_max_holding_hours,
        absolute_max_holding_hours: input.shadow_run.holding_policy.absolute_max_holding_hours,
        force_flat_policy: input.shadow_run.holding_policy.force_flat_policy.clone(),
        paper_start_recommendation: "start_paper_observation".to_owned(),
        schema_version: PAPER_TRADE_CANDIDATE_SCHEMA_VERSION.to_owned(),
    }
}
