use super::*;

pub(in crate::cli) async fn run_retest_refresh_cycle_mode(args: &Args) -> AppResult<RunSummary> {
    let output_partition_at_ms = args.now_ms.unwrap_or_else(now_ms);
    let manifest = load_input_manifest(args).await?.ok_or_else(|| {
        AppError::config(
            "--run-retest-refresh-cycle requires --input-manifest-file or S3 manifest input",
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
    let mut output_files =
        write_retest_refresh_cycle_plan_output(args, &plan, output_partition_at_ms).await?;

    let status = build_retest_horizon_status(
        &plan,
        None,
        &RetestHorizonStatusBuildOptions {
            generated_at_ms: output_partition_at_ms,
            plan_file: output_files.first().cloned(),
            driver_summary_file: None,
            checkpoint_s3_write: args.output_s3_bucket.is_some(),
        },
    )?;
    let validation = validate_retest_horizon_status(&status)?;
    output_files.extend(
        write_retest_refresh_cycle_status_output(args, &status, output_partition_at_ms).await?,
    );

    let mut retest_cycle_scheduler_action = validation.scheduler_action;
    let mut focused_retest_manifests_created = 0;
    let mut focused_retest_horizon_count = 0;
    let mut focused_retest_candidate_bundle_refs = 0;
    if retest_cycle_scheduler_action == "RUN_FOCUSED_RETEST_RESEARCH" {
        let mut build = build_focused_retest_manifest(
            &status,
            &manifest,
            &FocusedRetestBuildOptions {
                generated_at_ms: output_partition_at_ms,
                research_packet_id: focused_retest_packet_id(args, output_partition_at_ms),
                run_scope: focused_retest_run_scope(args),
                next_actions: args.focused_retest_next_actions.clone(),
                candidate_lifecycle_key_filter: Vec::new(),
                historical_replay_index_ref_mode: args
                    .focused_retest_historical_replay_index_ref_mode,
                s3_write: args.output_s3_bucket.is_some(),
            },
        )?;
        if args.output_s3_bucket.is_some() {
            let dispatch_packet_id =
                focused_retest_dispatch_packet_id(args, latest_l1_as_of_ms, &build)?;
            build.manifest.research_packet_id = Some(dispatch_packet_id);
        }
        focused_retest_horizon_count = build.summary.focused.focus_horizon_count;
        focused_retest_candidate_bundle_refs =
            build.summary.focused.selected_candidate_bundle_ref_count;
        let write_result = write_retest_refresh_cycle_focused_manifest_output(args, &build).await?;
        if write_result.created {
            focused_retest_manifests_created = 1;
        } else {
            retest_cycle_scheduler_action =
                "SKIP_FOCUSED_RETEST_RESEARCH_ALREADY_DISPATCHED".to_owned();
        }
        output_files.extend(write_result.output_files);
    }

    Ok(RunSummary {
        retest_horizon_plans_created: 1,
        retest_horizon_statuses_validated: 1,
        retest_cycle_scheduler_action: Some(retest_cycle_scheduler_action),
        retest_cycle_run_not_before_ms: validation.run_not_before_ms,
        focused_retest_manifests_created,
        focused_retest_horizon_count,
        focused_retest_candidate_bundle_refs,
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

pub(in crate::cli) async fn run_retest_refresh_cycle_from_latest_state_mode(
    args: &Args,
) -> AppResult<RunSummary> {
    let Some(bucket) = args.output_s3_bucket.as_deref() else {
        return Err(AppError::config(
            "--run-retest-refresh-cycle-from-latest-state requires --output-s3-bucket",
        ));
    };
    let state = read_latest_retest_cycle_source_state_from_s3(bucket, "").await?;
    let mut derived_args = args.clone();
    derived_args.run_retest_refresh_cycle_from_latest_state = false;
    derived_args.run_retest_refresh_cycle = true;
    derived_args.input_manifest_file = None;
    derived_args.input_manifest_s3_bucket = Some(state.source_manifest_s3_bucket);
    derived_args.input_manifest_s3_key = Some(state.source_manifest_s3_key);
    derived_args.research_report_file = None;
    derived_args.research_report_s3_bucket = Some(state.source_research_report_s3_bucket);
    derived_args.research_report_s3_key = Some(state.source_research_report_s3_key);
    run_retest_refresh_cycle_mode(&derived_args).await
}
