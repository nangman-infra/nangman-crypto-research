use super::super::*;

pub(in crate::cli) async fn build_focused_retest_manifest_mode(
    args: &Args,
) -> AppResult<RunSummary> {
    let status = load_retest_horizon_status(args).await?;
    build_focused_retest_manifest_from_status(args, &status, None).await
}

pub(in crate::cli) async fn build_focused_retest_manifest_from_status(
    args: &Args,
    status: &serde_json::Value,
    scheduler_action: Option<String>,
) -> AppResult<RunSummary> {
    let source_manifest = load_input_manifest(args).await?.ok_or_else(|| {
        AppError::config(
            "--build-focused-retest-manifest requires --input-manifest-file or S3 manifest input",
        )
    })?;
    validate_input_manifest(Some(&source_manifest))?;
    let output_partition_at_ms = args.now_ms.unwrap_or_else(now_ms);
    let build = build_focused_retest_manifest(
        status,
        &source_manifest,
        &FocusedRetestBuildOptions {
            generated_at_ms: output_partition_at_ms,
            research_packet_id: focused_retest_packet_id(args, output_partition_at_ms),
            run_scope: focused_retest_run_scope(args),
            next_actions: args.focused_retest_next_actions.clone(),
            candidate_lifecycle_key_filter: Vec::new(),
            historical_replay_index_ref_mode: args.focused_retest_historical_replay_index_ref_mode,
            s3_write: args.output_s3_bucket.is_some(),
        },
    )?;
    let focused_horizon_count = build.summary.focused.focus_horizon_count;
    let focused_candidate_bundle_refs = build.summary.focused.selected_candidate_bundle_ref_count;
    let output_files =
        write_focused_retest_manifest_outputs(args, &build, output_partition_at_ms).await?;

    Ok(RunSummary {
        retest_horizon_plans_created: 0,
        retest_horizon_statuses_validated: 1,
        retest_cycle_scheduler_action: scheduler_action,
        retest_cycle_run_not_before_ms: None,
        focused_retest_manifests_created: 1,
        focused_retest_horizon_count: focused_horizon_count,
        focused_retest_candidate_bundle_refs: focused_candidate_bundle_refs,
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
