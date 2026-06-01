use super::inputs::RetestRefreshCycleInputs;
use super::*;

pub(super) struct RetestRefreshCycleFocusedDispatch {
    pub(super) scheduler_action: String,
    pub(super) manifests_created: usize,
    pub(super) horizon_count: usize,
    pub(super) candidate_bundle_refs: usize,
    pub(super) output_files: Vec<String>,
}

pub(super) async fn maybe_write_focused_retest_manifest(
    args: &Args,
    inputs: &RetestRefreshCycleInputs,
    status: &serde_json::Value,
) -> AppResult<RetestRefreshCycleFocusedDispatch> {
    let scheduler_action = validate_retest_horizon_status(status)?.scheduler_action;
    if scheduler_action != "RUN_FOCUSED_RETEST_RESEARCH" {
        return Ok(RetestRefreshCycleFocusedDispatch {
            scheduler_action,
            manifests_created: 0,
            horizon_count: 0,
            candidate_bundle_refs: 0,
            output_files: Vec::new(),
        });
    }

    let mut build = build_focused_retest_manifest(
        status,
        &inputs.manifest,
        &FocusedRetestBuildOptions {
            generated_at_ms: inputs.output_partition_at_ms,
            research_packet_id: focused_retest_packet_id(args, inputs.output_partition_at_ms),
            run_scope: focused_retest_run_scope(args),
            next_actions: args.focused_retest_next_actions.clone(),
            candidate_lifecycle_key_filter: Vec::new(),
            historical_replay_index_ref_mode: args.focused_retest_historical_replay_index_ref_mode,
            s3_write: args.output_s3_bucket.is_some(),
        },
    )?;
    if args.output_s3_bucket.is_some() {
        let dispatch_packet_id =
            focused_retest_dispatch_packet_id(args, inputs.latest_l1_as_of_ms, &build)?;
        build.manifest.research_packet_id = Some(dispatch_packet_id);
    }

    let horizon_count = build.summary.focused.focus_horizon_count;
    let candidate_bundle_refs = build.summary.focused.selected_candidate_bundle_ref_count;
    let write_result = write_retest_refresh_cycle_focused_manifest_output(args, &build).await?;
    let scheduler_action = if write_result.created {
        "RUN_FOCUSED_RETEST_RESEARCH".to_owned()
    } else {
        "SKIP_FOCUSED_RETEST_RESEARCH_ALREADY_DISPATCHED".to_owned()
    };

    Ok(RetestRefreshCycleFocusedDispatch {
        scheduler_action,
        manifests_created: usize::from(write_result.created),
        horizon_count,
        candidate_bundle_refs,
        output_files: write_result.output_files,
    })
}
