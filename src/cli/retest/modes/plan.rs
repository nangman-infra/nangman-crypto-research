use super::super::*;

pub(in crate::cli) async fn build_retest_horizon_plan_mode(args: &Args) -> AppResult<RunSummary> {
    let manifest = load_input_manifest(args).await?.ok_or_else(|| {
        AppError::config(
            "--build-retest-horizon-plan requires --input-manifest-file or S3 manifest input",
        )
    })?;
    validate_input_manifest(Some(&manifest))?;
    let bundles = read_input_bundles(args, Some(&manifest)).await?;
    validate_manifest_budget(Some(&manifest), &manifest.runtime_budget_policy)?;
    enforce_budget(
        "candidate_bundle_count",
        bundles.len(),
        manifest.runtime_budget_policy.max_candidate_bundle_count,
    )?;
    let report = load_research_report(args).await?;
    let output_partition_at_ms = args.now_ms.unwrap_or_else(now_ms);
    let latest_l1_as_of_ms = retest_plan_latest_l1_as_of_ms(args).await?;
    let plan = build_retest_horizon_plan(
        &bundles,
        &report,
        &RetestHorizonPlanBuildOptions {
            generated_at_ms: output_partition_at_ms,
            manifest_label: input_manifest_label(args),
            report_label: research_report_label(args),
            latest_l1_as_of_ms,
        },
    )?;
    let output_files =
        write_retest_horizon_plan_outputs(args, &plan, output_partition_at_ms).await?;

    Ok(RunSummary {
        retest_horizon_plans_created: 1,
        retest_horizon_statuses_validated: 0,
        retest_cycle_scheduler_action: None,
        retest_cycle_run_not_before_ms: None,
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
