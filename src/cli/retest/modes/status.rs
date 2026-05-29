use super::super::*;

pub(in crate::cli) async fn build_retest_horizon_status_mode(args: &Args) -> AppResult<RunSummary> {
    let plan = load_retest_horizon_plan(args).await?;
    let driver_summary = load_retest_driver_summary(args)?;
    let output_partition_at_ms = args.now_ms.unwrap_or_else(now_ms);
    let status = build_retest_horizon_status(
        &plan,
        driver_summary.as_ref(),
        &RetestHorizonStatusBuildOptions {
            generated_at_ms: output_partition_at_ms,
            plan_file: args
                .retest_horizon_plan_file
                .as_ref()
                .map(|path| path.display().to_string()),
            driver_summary_file: args
                .retest_driver_summary_file
                .as_ref()
                .map(|path| path.display().to_string()),
            checkpoint_s3_write: args.output_s3_bucket.is_some(),
        },
    )?;
    let validation = validate_retest_horizon_status(&status)?;
    let output_files =
        write_retest_horizon_status_outputs(args, &status, output_partition_at_ms).await?;

    Ok(RunSummary {
        retest_horizon_plans_created: 0,
        retest_horizon_statuses_validated: 1,
        retest_cycle_scheduler_action: Some(validation.scheduler_action),
        retest_cycle_run_not_before_ms: validation.run_not_before_ms,
        focused_retest_manifests_created: 0,
        focused_retest_horizon_count: 0,
        focused_retest_candidate_bundle_refs: 0,
        shadow_cycle_decisions_validated: 0,
        shadow_cycle_decisions_created: 0,
        shadow_cycle_scheduler_action: None,
        shadow_cycle_run_not_before_ms: None,
        shadow_cycle_focused_research_manifest_file: None,
        processed_bundles: 0,
        replay_runs_created: 0,
        historical_replay_runs_loaded: 0,
        oss_adapter_runs_loaded: 0,
        shadow_validation_runs_loaded: 0,
        shadow_validation_runs_created: 0,
        paper_trade_candidates_created: 0,
        paper_trade_runs_created: 0,
        paper_trade_summaries_created: 0,
        paper_trade_marks_created: 0,
        paper_watch_live_marks_created: 0,
        paper_watch_observer_iterations: 0,
        paper_watch_observer_snapshots_created: 0,
        paper_watch_observer_active_candidates: 0,
        paper_watch_observer_restored_live_marks: 0,
        portfolio_risk_reject_events_created: 0,
        portfolio_reduce_only_signals_created: 0,
        output_files,
    })
}
