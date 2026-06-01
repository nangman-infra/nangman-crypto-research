use super::*;

pub(super) fn shadow_accumulation_dispatch_packet_id(
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
