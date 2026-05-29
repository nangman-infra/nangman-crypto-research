use crate::model::{
    PaperTradeCandidate, PaperTradeMark, PaperTradeRun, PaperTradeSummary, PaperWatchCandidate,
    ReplayRun, ResearchRunReport, ShadowValidationRun,
};

pub type PortfolioOutputBodies = (Option<Vec<u8>>, Vec<u8>, Vec<u8>);

pub struct ResearchOutputArtifacts<'a> {
    pub report: &'a ResearchRunReport,
    pub replay_runs: &'a [ReplayRun],
    pub shadow_validation_runs: &'a [ShadowValidationRun],
    pub paper_watch_candidates: &'a [PaperWatchCandidate],
    pub paper_trade_candidates: &'a [PaperTradeCandidate],
    pub paper_trade_runs: &'a [PaperTradeRun],
    pub paper_trade_summaries: &'a [PaperTradeSummary],
    pub paper_trade_marks: &'a [PaperTradeMark],
    pub output_partition_at_ms: i64,
}
