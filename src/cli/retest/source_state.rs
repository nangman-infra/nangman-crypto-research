use super::*;

pub(in crate::cli) async fn write_retest_cycle_source_state_output(
    args: &Args,
    report: &crate::model::ResearchRunReport,
    output_files: &[String],
    output_partition_at_ms: i64,
) -> AppResult<Vec<String>> {
    let Some(output_bucket) = args.output_s3_bucket.as_deref() else {
        return Ok(Vec::new());
    };
    let (Some(source_manifest_s3_bucket), Some(source_manifest_s3_key)) = (
        args.input_manifest_s3_bucket.as_deref(),
        args.input_manifest_s3_key.as_deref(),
    ) else {
        return Ok(Vec::new());
    };
    let source_research_report_s3_key = research_report_s3_key_from_output_files(
        output_bucket,
        output_files,
        &report.research_run_report_id,
    )?;
    let state = build_retest_cycle_source_state(
        output_partition_at_ms,
        source_manifest_s3_bucket,
        source_manifest_s3_key,
        output_bucket,
        &source_research_report_s3_key,
        report,
    );
    write_retest_cycle_source_state_to_s3(output_bucket, "", &state, output_partition_at_ms)
        .await
        .map(|uri| vec![uri])
}

pub(in crate::cli) fn build_retest_cycle_source_state(
    generated_at_ms: i64,
    source_manifest_s3_bucket: &str,
    source_manifest_s3_key: &str,
    source_research_report_s3_bucket: &str,
    source_research_report_s3_key: &str,
    report: &crate::model::ResearchRunReport,
) -> RetestCycleSourceState {
    RetestCycleSourceState {
        schema_version: RETEST_CYCLE_SOURCE_STATE_SCHEMA_VERSION.to_owned(),
        generated_at_ms,
        research_packet_id: report.research_packet_id.clone(),
        run_scope: report.run_scope.clone(),
        source_manifest_s3_bucket: source_manifest_s3_bucket.to_owned(),
        source_manifest_s3_key: source_manifest_s3_key.to_owned(),
        source_research_report_s3_bucket: source_research_report_s3_bucket.to_owned(),
        source_research_report_s3_key: source_research_report_s3_key.to_owned(),
        source_research_report_id: report.research_run_report_id.clone(),
        source_candidate_ids: report.source_candidate_ids.clone(),
        replay_run_id_count: report.replay_run_ids.len(),
        summary_findings_count: report.summary_findings.len(),
        shadow_validation_run_count: report.shadow_validation_runs.len(),
        paper_trade_candidate_count: report.paper_trade_candidates.len(),
        safety: RetestCycleSourceStateSafety {
            dispatcher_prefix: "research-input-manifest/".to_owned(),
            state_s3_write: true,
            ecs_task_started: false,
            shadow_paper_live_enabled: false,
        },
    }
}

pub(in crate::cli) fn research_report_s3_key_from_output_files(
    bucket: &str,
    output_files: &[String],
    research_run_report_id: &str,
) -> AppResult<String> {
    let uri_prefix = format!("s3://{bucket}/");
    let report_path = format!("research_run_report_id={research_run_report_id}/report.json");
    output_files
        .iter()
        .find_map(|file| {
            file.strip_prefix(&uri_prefix)
                .filter(|key| {
                    key.starts_with("research-run-report/")
                        || key.contains("/research-run-report/")
                })
                .filter(|key| key.ends_with(&report_path))
                .map(ToOwned::to_owned)
        })
        .ok_or_else(|| {
            AppError::validation(format!(
                "research output files missing S3 report for research_run_report_id={research_run_report_id}"
            ))
        })
}
