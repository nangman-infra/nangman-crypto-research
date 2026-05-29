use crate::model::{PaperTradeCandidate, PaperTradeMark, PaperTradeRun, PaperTradeSummary};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PaperArtifacts {
    pub candidates: Vec<PaperTradeCandidate>,
    pub runs: Vec<PaperTradeRun>,
    pub summaries: Vec<PaperTradeSummary>,
    pub marks: Vec<PaperTradeMark>,
}
