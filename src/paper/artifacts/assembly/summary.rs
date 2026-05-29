use super::super::outcome::promote_recommendation;
use super::CandidatePaperBuildInput;
use crate::hash::stable_id;
use crate::model::{
    PAPER_TRADE_SUMMARY_SCHEMA_VERSION, PaperTradeCandidate, PaperTradeRun, PaperTradeSummary,
};

pub(super) fn build_summary(
    input: &CandidatePaperBuildInput<'_>,
    paper_trade_run_id: &str,
    candidate: &PaperTradeCandidate,
    run: &PaperTradeRun,
    survival_result: &str,
) -> PaperTradeSummary {
    PaperTradeSummary {
        paper_trade_summary_id: stable_id("paper_trade_summary", &[paper_trade_run_id]),
        paper_trade_run_id: paper_trade_run_id.to_owned(),
        candidate_lifecycle_key: input.candidate_lifecycle_key.to_owned(),
        summary_window: format!(
            "target_{}h_absolute_{}h",
            input.shadow_run.holding_policy.target_max_holding_hours,
            input.shadow_run.holding_policy.absolute_max_holding_hours
        ),
        entry_behavior_summary: format!(
            "entries_from_completed_replay_windows={}",
            run.entry_count
        ),
        exit_behavior_summary: format!("ttl_exit_policy={}", candidate.force_flat_policy),
        cost_behavior_summary: format!(
            "fee_model={},slippage_model={},estimated_cost_bps={}",
            input.profile.fee_model_version,
            input.profile.slippage_model_version,
            input
                .aggregate
                .estimated_cost_bps
                .map(|value| value.to_string())
                .unwrap_or_else(|| "unknown".to_owned())
        ),
        risk_behavior_summary: format!(
            "survival_band={:?},max_drawdown_band={}",
            input.aggregate.survival_band, run.max_drawdown_band
        ),
        promote_recommendation: promote_recommendation(survival_result),
        schema_version: PAPER_TRADE_SUMMARY_SCHEMA_VERSION.to_owned(),
    }
}
