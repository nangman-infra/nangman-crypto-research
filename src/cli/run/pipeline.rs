use super::outputs::write_research_pipeline_outputs;
use super::research_inputs::{ResearchInputs, load_research_inputs};
use super::*;
use crate::model::{ReplayRun, ResearchRunReport};
use crate::paper::PaperArtifacts;

mod identity;
mod replay;
mod summary;

use identity::resolve_research_identity;
use replay::{aggregate_replay_runs, load_current_historical_replay_runs};
use summary::build_run_summary;

pub(super) async fn run_research_pipeline(args: &Args) -> AppResult<RunSummary> {
    let inputs = load_research_inputs(args).await?;
    let created_at_ms = args
        .now_ms
        .unwrap_or_else(|| deterministic_report_created_at_ms(&inputs.bundles));
    let output_partition_at_ms = args.now_ms.unwrap_or_else(now_ms);
    let replay_runs = build_replay_runs(
        &inputs.bundles,
        &inputs.market_deltas,
        &inputs.regime_contexts,
    );
    enforce_budget(
        "new_replay_run_count",
        replay_runs.len(),
        inputs.budget.max_replay_run_count,
    )?;
    let historical_replay_runs =
        load_current_historical_replay_runs(args, &inputs, &replay_runs).await?;
    let aggregate_replay_runs =
        aggregate_replay_runs(&inputs, &replay_runs, &historical_replay_runs)?;
    let (research_packet_id, run_scope) = resolve_research_identity(args, &inputs);
    let mut report = build_report(
        research_packet_id,
        run_scope,
        created_at_ms,
        &inputs.bundles,
        &aggregate_replay_runs,
        &inputs.oss_adapter_runs,
        &inputs.shadow_validation_runs,
    );
    let paper_watch_candidates =
        build_paper_watch_candidates(&report, &inputs.bundles, created_at_ms);
    report.paper_watch_candidates = paper_watch_candidates
        .iter()
        .map(|candidate| candidate.paper_watch_candidate_id.clone())
        .collect();
    let paper_artifacts = build_paper_artifacts(
        &report,
        &inputs.bundles,
        &inputs.shadow_validation_runs,
        created_at_ms,
    );
    report.paper_trade_candidates = paper_artifacts
        .candidates
        .iter()
        .map(|candidate| candidate.paper_trade_candidate_id.clone())
        .collect();
    let output_artifacts = ResearchOutputArtifacts {
        report: &report,
        replay_runs: &replay_runs,
        shadow_validation_runs: &report.shadow_validation_runs,
        paper_watch_candidates: &paper_watch_candidates,
        paper_trade_candidates: &paper_artifacts.candidates,
        paper_trade_runs: &paper_artifacts.runs,
        paper_trade_summaries: &paper_artifacts.summaries,
        paper_trade_marks: &paper_artifacts.marks,
        output_partition_at_ms,
    };
    let output_files =
        write_research_pipeline_outputs(args, &report, &output_artifacts, output_partition_at_ms)
            .await?;
    emit_research_report_alert_from_env(&report, &paper_watch_candidates).await;

    Ok(build_run_summary(
        &inputs,
        &replay_runs,
        &historical_replay_runs,
        &report,
        &paper_artifacts,
        output_files,
    ))
}
