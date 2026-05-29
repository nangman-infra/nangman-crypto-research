use super::CandidatePaperBuildInput;
use crate::model::{PAPER_TRADE_RUN_SCHEMA_VERSION, PaperTradeRun};
use crate::paper::shared::max_drawdown_band;

const MS_PER_HOUR: i64 = 60 * 60 * 1000;

pub(super) fn build_run(
    input: &CandidatePaperBuildInput<'_>,
    paper_trade_run_id: &str,
    net_result_band: &str,
    survival_result: &str,
) -> PaperTradeRun {
    PaperTradeRun {
        paper_trade_run_id: paper_trade_run_id.to_owned(),
        candidate_lifecycle_key: input.candidate_lifecycle_key.to_owned(),
        symbol_canonical: input.aggregate.symbol_canonical.clone(),
        source_research_run_id: input.report.research_run_report_id.clone(),
        paper_account_profile_id: input.profile.paper_account_profile_id.clone(),
        started_at_ms: input.created_at_ms,
        ended_at_ms: input.created_at_ms
            + i64::from(input.shadow_run.holding_policy.target_max_holding_hours) * MS_PER_HOUR,
        entry_count: input.aggregate.completed_count,
        exit_count: input.aggregate.completed_count,
        max_drawdown_band: max_drawdown_band(input.aggregate),
        net_result_band: net_result_band.to_owned(),
        survival_result: survival_result.to_owned(),
        schema_version: PAPER_TRADE_RUN_SCHEMA_VERSION.to_owned(),
    }
}
