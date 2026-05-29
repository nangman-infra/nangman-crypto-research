mod candidate;
mod mark;
mod run;
mod summary;

use crate::hash::stable_id;
use crate::model::{
    PaperAccountProfile, PaperTradeCandidate, PaperTradeMark, PaperTradeRun, PaperTradeSummary,
    ResearchPartitionAggregate, ResearchRunReport, ShadowValidationRun,
};

use self::candidate::build_candidate;
use self::mark::build_mark;
use self::run::build_run;
use self::summary::build_summary;
use super::outcome::{net_result_band, survival_result};

pub(super) struct CandidatePaperArtifacts {
    pub(super) candidate: PaperTradeCandidate,
    pub(super) run: PaperTradeRun,
    pub(super) summary: PaperTradeSummary,
    pub(super) mark: PaperTradeMark,
}

pub(super) struct CandidatePaperBuildInput<'a> {
    pub(super) report: &'a ResearchRunReport,
    pub(super) candidate_lifecycle_key: &'a str,
    pub(super) aggregate: &'a ResearchPartitionAggregate,
    pub(super) shadow_run: &'a ShadowValidationRun,
    pub(super) profile: &'a PaperAccountProfile,
    pub(super) created_at_ms: i64,
}

pub(super) fn build_candidate_paper_artifacts(
    input: CandidatePaperBuildInput<'_>,
) -> CandidatePaperArtifacts {
    let paper_candidate_id = stable_id(
        "paper_trade_candidate",
        &[
            &input.report.research_run_report_id,
            input.candidate_lifecycle_key,
            &input.shadow_run.shadow_validation_run_id,
        ],
    );
    let candidate = build_candidate(&input, &paper_candidate_id);
    let paper_trade_run_id = stable_id(
        "paper_trade_run",
        &[&paper_candidate_id, &input.report.research_run_report_id],
    );
    let net_result_band = net_result_band(input.aggregate);
    let survival_result = survival_result(input.aggregate);
    let run = build_run(
        &input,
        &paper_trade_run_id,
        &net_result_band,
        &survival_result,
    );
    let summary = build_summary(
        &input,
        &paper_trade_run_id,
        &candidate,
        &run,
        &survival_result,
    );
    let mark = build_mark(&input, paper_trade_run_id, net_result_band, survival_result);

    CandidatePaperArtifacts {
        candidate,
        run,
        summary,
        mark,
    }
}
