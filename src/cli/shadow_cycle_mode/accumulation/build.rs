use super::packet_id::shadow_accumulation_dispatch_packet_id;
use super::types::ShadowAccumulationManifestDispatch;
use super::*;

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
