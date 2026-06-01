use super::super::super::{
    AppError, AppResult, Args, write_pretty_json_file, write_retest_horizon_plan_to_s3,
    write_retest_horizon_status_to_s3,
};

pub(in crate::cli) async fn write_retest_horizon_plan_outputs(
    args: &Args,
    plan: &serde_json::Value,
    output_partition_at_ms: i64,
) -> AppResult<Vec<String>> {
    if let Some(path) = args.retest_horizon_plan_output_file.as_deref() {
        return Ok(vec![
            write_pretty_json_file(path, plan)?.display().to_string(),
        ]);
    }
    let Some(bucket) = args.output_s3_bucket.as_deref() else {
        return Err(AppError::config(
            "--build-retest-horizon-plan requires --retest-horizon-plan-output-file or --output-s3-bucket",
        ));
    };
    let uri = write_retest_horizon_plan_to_s3(
        bucket,
        args.output_s3_prefix.as_deref().unwrap_or(""),
        plan,
        output_partition_at_ms,
    )
    .await?;
    Ok(vec![uri])
}

pub(in crate::cli) async fn write_retest_horizon_status_outputs(
    args: &Args,
    status: &serde_json::Value,
    output_partition_at_ms: i64,
) -> AppResult<Vec<String>> {
    if let Some(path) = args.retest_horizon_status_output_file.as_deref() {
        return Ok(vec![
            write_pretty_json_file(path, status)?.display().to_string(),
        ]);
    }
    let Some(bucket) = args.output_s3_bucket.as_deref() else {
        return Err(AppError::config(
            "--build-retest-horizon-status requires --retest-horizon-status-output-file or --output-s3-bucket",
        ));
    };
    let uri = write_retest_horizon_status_to_s3(
        bucket,
        args.output_s3_prefix.as_deref().unwrap_or(""),
        status,
        output_partition_at_ms,
    )
    .await?;
    Ok(vec![uri])
}
