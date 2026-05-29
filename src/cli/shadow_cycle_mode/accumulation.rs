use super::*;

#[derive(Debug)]
pub(super) struct ShadowAccumulationDispatch {
    pub(super) manifest_uri: String,
    pub(super) created: bool,
    pub(super) focused_horizon_count: usize,
    pub(super) focused_candidate_bundle_refs: usize,
    pub(super) deficit_lifecycle_keys: Vec<String>,
}

#[derive(Debug)]
pub(in crate::cli) struct ShadowAccumulationManifestDispatch {
    pub(in crate::cli) key: String,
    pub(in crate::cli) manifest: ResearchInputManifest,
    pub(in crate::cli) focused_horizon_count: usize,
    pub(in crate::cli) focused_candidate_bundle_refs: usize,
    pub(in crate::cli) deficit_lifecycle_keys: Vec<String>,
}

pub(super) async fn try_build_shadow_accumulation_manifest_from_latest_state(
    args: &Args,
    shadow_runs: &[ShadowValidationRun],
    latest_l1_as_of_ms: Option<i64>,
    output_partition_at_ms: i64,
) -> AppResult<Option<ShadowAccumulationDispatch>> {
    let deficit_lifecycle_keys =
        shadow_sample_deficit_lifecycle_keys(shadow_runs, latest_l1_as_of_ms);
    let Some(bucket) = args.output_s3_bucket.as_deref() else {
        return Ok(None);
    };
    let state = match read_latest_retest_cycle_source_state_from_s3(bucket, "").await {
        Ok(state) => state,
        Err(AppError::AwsNotFound(_)) => return Ok(None),
        Err(error) => return Err(error),
    };
    let status = match read_latest_retest_horizon_status_from_s3(bucket, "").await {
        Ok(status) => status,
        Err(AppError::AwsNotFound(_)) => return Ok(None),
        Err(error) => return Err(error),
    };
    let source_manifest = read_research_input_manifest_from_s3(
        &state.source_manifest_s3_bucket,
        &state.source_manifest_s3_key,
    )
    .await?;
    validate_input_manifest(Some(&source_manifest))?;

    let Some(dispatch_build) = build_shadow_accumulation_manifest_dispatch(
        args,
        &state,
        &status,
        &source_manifest,
        latest_l1_as_of_ms,
        output_partition_at_ms,
        deficit_lifecycle_keys,
    )?
    else {
        return Ok(None);
    };

    let write_result = write_research_input_manifest_to_exact_s3_key_if_absent(
        bucket,
        &dispatch_build.key,
        &dispatch_build.manifest,
    )
    .await?;
    let created = write_result.is_some();
    let manifest_uri =
        write_result.unwrap_or_else(|| format!("s3://{bucket}/{}", dispatch_build.key));

    Ok(Some(ShadowAccumulationDispatch {
        manifest_uri,
        created,
        focused_horizon_count: dispatch_build.focused_horizon_count,
        focused_candidate_bundle_refs: dispatch_build.focused_candidate_bundle_refs,
        deficit_lifecycle_keys: dispatch_build.deficit_lifecycle_keys,
    }))
}

pub(in crate::cli) fn build_shadow_accumulation_manifest_dispatch(
    args: &Args,
    state: &RetestCycleSourceState,
    status: &serde_json::Value,
    source_manifest: &ResearchInputManifest,
    latest_l1_as_of_ms: Option<i64>,
    output_partition_at_ms: i64,
    deficit_lifecycle_keys: Vec<String>,
) -> AppResult<Option<ShadowAccumulationManifestDispatch>> {
    if deficit_lifecycle_keys.is_empty() {
        return Ok(None);
    }
    let mut build = match build_focused_retest_manifest(
        status,
        source_manifest,
        &FocusedRetestBuildOptions {
            generated_at_ms: output_partition_at_ms,
            research_packet_id: "research_shadow_accumulation_pending".to_owned(),
            run_scope: "shadow_sample_accumulation_local_validation".to_owned(),
            next_actions: args.focused_retest_next_actions.clone(),
            candidate_lifecycle_key_filter: deficit_lifecycle_keys.clone(),
            historical_replay_index_ref_mode: args.focused_retest_historical_replay_index_ref_mode,
            s3_write: true,
        },
    ) {
        Ok(build) => build,
        Err(AppError::Validation(message))
            if message.contains("selected zero candidate bundle refs") =>
        {
            return Ok(None);
        }
        Err(error) => return Err(error),
    };
    let packet_id = shadow_accumulation_dispatch_packet_id(
        state,
        latest_l1_as_of_ms,
        &deficit_lifecycle_keys,
        &build,
    )?;
    build.manifest.research_packet_id = Some(packet_id.clone());
    let key = focused_retest_dispatch_manifest_s3_key(&packet_id)?;

    Ok(Some(ShadowAccumulationManifestDispatch {
        key,
        manifest: build.manifest,
        focused_horizon_count: build.summary.focused.focus_horizon_count,
        focused_candidate_bundle_refs: build.summary.focused.selected_candidate_bundle_ref_count,
        deficit_lifecycle_keys,
    }))
}

fn shadow_accumulation_dispatch_packet_id(
    state: &RetestCycleSourceState,
    latest_l1_as_of_ms: Option<i64>,
    deficit_lifecycle_keys: &[String],
    build: &FocusedRetestManifestBuild,
) -> AppResult<String> {
    let latest_l1_part = latest_l1_as_of_ms
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_owned());
    let focus_rows = serde_json::to_string(&build.summary.focused.rows)?;
    let candidate_refs = serde_json::to_string(&build.manifest.candidate_bundle_refs)?;
    let historical_index_refs =
        serde_json::to_string(&build.manifest.historical_replay_run_index_refs)?;
    let deficit_keys = serde_json::to_string(deficit_lifecycle_keys)?;
    let parts = [
        "research_shadow_accumulation_dispatch_v1",
        state.source_manifest_s3_key.as_str(),
        state.source_research_report_s3_key.as_str(),
        latest_l1_part.as_str(),
        deficit_keys.as_str(),
        focus_rows.as_str(),
        candidate_refs.as_str(),
        historical_index_refs.as_str(),
    ];
    Ok(stable_id("research_shadow_accumulation", &parts))
}
