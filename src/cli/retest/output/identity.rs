use super::super::*;

pub(in crate::cli) fn focused_retest_summary_output_path(
    args: &Args,
    manifest_output_file: &std::path::Path,
) -> PathBuf {
    args.focused_retest_summary_output_file
        .clone()
        .unwrap_or_else(|| {
            PathBuf::from(format!("{}.summary.json", manifest_output_file.display()))
        })
}

pub(in crate::cli) fn focused_retest_packet_id(args: &Args, output_partition_at_ms: i64) -> String {
    if args.research_packet_id == "local_research_packet" {
        format!("research_focus_{output_partition_at_ms}")
    } else {
        args.research_packet_id.clone()
    }
}

pub(in crate::cli) fn focused_retest_dispatch_packet_id(
    args: &Args,
    latest_l1_as_of_ms: Option<i64>,
    build: &FocusedRetestManifestBuild,
) -> AppResult<String> {
    let latest_l1_part = latest_l1_as_of_ms
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_owned());
    let focus_next_actions = serde_json::to_string(&build.summary.focus_next_actions)?;
    let focus_rows = serde_json::to_string(&build.summary.focused.rows)?;
    let candidate_refs = serde_json::to_string(&build.manifest.candidate_bundle_refs)?;
    let historical_index_refs =
        serde_json::to_string(&build.manifest.historical_replay_run_index_refs)?;
    let manifest_label = input_manifest_label(args);
    let report_label = research_report_label(args);
    let run_scope = focused_retest_run_scope(args);
    let parts = [
        "research_retest_refresh_focused_dispatch_v1".to_owned(),
        manifest_label,
        report_label,
        latest_l1_part,
        run_scope,
        focus_next_actions,
        focus_rows,
        candidate_refs,
        historical_index_refs,
    ];
    let part_refs = parts.iter().map(String::as_str).collect::<Vec<_>>();
    Ok(stable_id("research_focus", &part_refs))
}

pub(in crate::cli) fn focused_retest_dispatch_manifest_s3_key(
    packet_id: &str,
) -> AppResult<String> {
    let packet_id = packet_id.trim();
    if packet_id.is_empty() {
        return Err(AppError::validation(
            "focused retest dispatch packet id must not be empty",
        ));
    }
    if packet_id.contains('/') {
        return Err(AppError::validation(
            "focused retest dispatch packet id must not contain /",
        ));
    }
    Ok(format!(
        "research-input-manifest/schema=research_input_manifest_v1/dedupe_key={packet_id}/manifest.json"
    ))
}

pub(in crate::cli) fn focused_retest_run_scope(args: &Args) -> String {
    if args.run_scope == "p0_candidate_bundle_local" {
        "focused_retest_local_validation".to_owned()
    } else {
        args.run_scope.clone()
    }
}
