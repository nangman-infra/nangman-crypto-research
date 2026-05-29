use super::super::*;

pub(in crate::cli) async fn write_retest_refresh_cycle_plan_output(
    args: &Args,
    plan: &serde_json::Value,
    output_partition_at_ms: i64,
) -> AppResult<Vec<String>> {
    write_retest_refresh_cycle_checkpoint_output(
        args,
        plan,
        output_partition_at_ms,
        RetestRefreshCheckpointKind::Plan,
    )
    .await
}

pub(in crate::cli) async fn write_retest_refresh_cycle_status_output(
    args: &Args,
    status: &serde_json::Value,
    output_partition_at_ms: i64,
) -> AppResult<Vec<String>> {
    write_retest_refresh_cycle_checkpoint_output(
        args,
        status,
        output_partition_at_ms,
        RetestRefreshCheckpointKind::Status,
    )
    .await
}

enum RetestRefreshCheckpointKind {
    Plan,
    Status,
}

impl RetestRefreshCheckpointKind {
    fn local_filename(&self) -> &'static str {
        match self {
            Self::Plan => "retest-horizon-plan.json",
            Self::Status => "retest-horizon-status.json",
        }
    }

    fn s3_prefix(&self) -> &'static str {
        match self {
            Self::Plan => "retest-horizon-plan/schema=research_retest_horizon_plan_v1",
            Self::Status => "retest-horizon-status/schema=research_horizon_status_checkpoint_v1",
        }
    }
}

async fn write_retest_refresh_cycle_checkpoint_output(
    args: &Args,
    value: &serde_json::Value,
    output_partition_at_ms: i64,
    kind: RetestRefreshCheckpointKind,
) -> AppResult<Vec<String>> {
    if let Some(output_dir) = args.output_dir.as_deref() {
        let path = output_dir.join(kind.local_filename());
        return Ok(vec![
            write_pretty_json_file(&path, value)?.display().to_string(),
        ]);
    }
    let Some(bucket) = args.output_s3_bucket.as_deref() else {
        return Err(AppError::config(
            "--run-retest-refresh-cycle requires --output-dir or --output-s3-bucket",
        ));
    };
    let uri = match kind {
        RetestRefreshCheckpointKind::Plan => {
            write_retest_horizon_plan_to_s3(bucket, kind.s3_prefix(), value, output_partition_at_ms)
                .await?
        }
        RetestRefreshCheckpointKind::Status => {
            write_retest_horizon_status_to_s3(
                bucket,
                kind.s3_prefix(),
                value,
                output_partition_at_ms,
            )
            .await?
        }
    };
    Ok(vec![uri])
}

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
