use super::*;

pub(in crate::cli) async fn load_retest_horizon_status(
    args: &Args,
) -> AppResult<serde_json::Value> {
    match (
        args.retest_horizon_status_file.as_deref(),
        args.retest_horizon_status_s3_bucket.as_deref(),
        args.retest_horizon_status_s3_key.as_deref(),
    ) {
        (Some(path), None, None) => read_retest_horizon_status(path),
        (None, Some(bucket), Some(key)) => read_retest_horizon_status_from_s3(bucket, key).await,
        _ => Err(AppError::config(
            "provide either --retest-horizon-status-file or --retest-horizon-status-s3-bucket/--retest-horizon-status-s3-key",
        )),
    }
}

pub(in crate::cli) async fn load_retest_horizon_plan(args: &Args) -> AppResult<serde_json::Value> {
    match (
        args.retest_horizon_plan_file.as_deref(),
        args.retest_horizon_plan_s3_bucket.as_deref(),
        args.retest_horizon_plan_s3_key.as_deref(),
    ) {
        (Some(path), None, None) => read_retest_horizon_plan(path),
        (None, Some(bucket), Some(key)) => read_retest_horizon_plan_from_s3(bucket, key).await,
        _ => Err(AppError::config(
            "provide either --retest-horizon-plan-file or --retest-horizon-plan-s3-bucket/--retest-horizon-plan-s3-key",
        )),
    }
}

pub(in crate::cli) async fn load_research_report(
    args: &Args,
) -> AppResult<crate::model::ResearchRunReport> {
    match (
        args.research_report_file.as_deref(),
        args.research_report_s3_bucket.as_deref(),
        args.research_report_s3_key.as_deref(),
    ) {
        (Some(path), None, None) => read_research_run_report(path),
        (None, Some(bucket), Some(key)) => read_research_run_report_from_s3(bucket, key).await,
        _ => Err(AppError::config(
            "provide either --research-report-file or --research-report-s3-bucket/--research-report-s3-key",
        )),
    }
}

pub(in crate::cli) async fn retest_plan_latest_l1_as_of_ms(args: &Args) -> AppResult<Option<i64>> {
    if let Some(latest_l1_as_of_ms) = args.retest_horizon_latest_l1_as_of_ms {
        return Ok(Some(latest_l1_as_of_ms));
    }
    let Some(bucket) = args.market_l1_s3_bucket.as_deref() else {
        return Ok(None);
    };
    if bucket.contains('<') || bucket.contains('>') {
        return Ok(None);
    }
    discover_latest_symbol_universe_snapshot_end_ms_from_s3(bucket).await
}

pub(in crate::cli) async fn shadow_cycle_latest_l1_as_of_ms(args: &Args) -> AppResult<Option<i64>> {
    if let Some(latest_l1_as_of_ms) = args.shadow_cycle_latest_l1_as_of_ms {
        return Ok(Some(latest_l1_as_of_ms));
    }
    let Some(bucket) = args.market_l1_s3_bucket.as_deref() else {
        return Ok(None);
    };
    if bucket.contains('<') || bucket.contains('>') {
        return Ok(None);
    }
    discover_latest_symbol_universe_snapshot_end_ms_from_s3(bucket).await
}

pub(in crate::cli) fn input_manifest_label(args: &Args) -> String {
    if let Some(path) = args.input_manifest_file.as_deref() {
        return path.display().to_string();
    }
    match (
        args.input_manifest_s3_bucket.as_deref(),
        args.input_manifest_s3_key.as_deref(),
    ) {
        (Some(bucket), Some(key)) => format!("s3://{bucket}/{key}"),
        _ => "unknown".to_owned(),
    }
}

pub(in crate::cli) fn research_report_label(args: &Args) -> String {
    if let Some(path) = args.research_report_file.as_deref() {
        return path.display().to_string();
    }
    match (
        args.research_report_s3_bucket.as_deref(),
        args.research_report_s3_key.as_deref(),
    ) {
        (Some(bucket), Some(key)) => format!("s3://{bucket}/{key}"),
        _ => "unknown".to_owned(),
    }
}

pub(in crate::cli) fn load_retest_driver_summary(
    args: &Args,
) -> AppResult<Option<serde_json::Value>> {
    let Some(path) = args.retest_driver_summary_file.as_deref() else {
        return Ok(None);
    };
    if !path.is_absolute() {
        return Err(AppError::config(
            "retest driver summary file must be an absolute path",
        ));
    }
    let bytes = fs::read(path)?;
    let value = serde_json::from_slice(&bytes)?;
    Ok(Some(value))
}
