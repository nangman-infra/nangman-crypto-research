use super::{PaperArtifacts, ReplayRun, ResearchInputs, ResearchRunReport, RunSummary};

pub(super) fn build_run_summary(
    inputs: &ResearchInputs,
    replay_runs: &[ReplayRun],
    historical_replay_runs: &[ReplayRun],
    report: &ResearchRunReport,
    paper_artifacts: &PaperArtifacts,
    output_files: Vec<String>,
) -> RunSummary {
    RunSummary {
        processed_bundles: inputs.bundles.len(),
        replay_runs_created: replay_runs.len(),
        historical_replay_runs_loaded: historical_replay_runs.len(),
        oss_adapter_runs_loaded: inputs.oss_adapter_runs.len(),
        shadow_validation_runs_loaded: inputs.shadow_validation_runs.len(),
        shadow_validation_runs_created: report.shadow_validation_runs.len(),
        paper_trade_candidates_created: paper_artifacts.candidates.len(),
        paper_trade_runs_created: paper_artifacts.runs.len(),
        paper_trade_summaries_created: paper_artifacts.summaries.len(),
        paper_trade_marks_created: paper_artifacts.marks.len(),
        portfolio_risk_reject_events_created: report.portfolio_risk_reject_events.len(),
        portfolio_reduce_only_signals_created: report.portfolio_reduce_only_signals.len(),
        output_files,
        ..RunSummary::default()
    }
}
