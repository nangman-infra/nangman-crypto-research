use super::CandidatePaperBuildInput;
use crate::hash::stable_id;
use crate::model::{PAPER_TRADE_MARK_SCHEMA_VERSION, PaperTradeMark};

pub(super) fn build_mark(
    input: &CandidatePaperBuildInput<'_>,
    paper_trade_run_id: String,
    net_result_band: String,
    survival_result: String,
) -> PaperTradeMark {
    PaperTradeMark {
        paper_trade_mark_id: stable_id("paper_trade_mark", &[&paper_trade_run_id]),
        paper_trade_run_id,
        candidate_lifecycle_key: input.candidate_lifecycle_key.to_owned(),
        symbol_canonical: input.aggregate.symbol_canonical.clone(),
        marked_at_ms: input.created_at_ms,
        mark_source: "research_replay_proxy".to_owned(),
        net_result_band,
        survival_result,
        schema_version: PAPER_TRADE_MARK_SCHEMA_VERSION.to_owned(),
    }
}
