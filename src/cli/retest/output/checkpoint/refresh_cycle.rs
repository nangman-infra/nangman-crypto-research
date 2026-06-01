use super::super::super::{
    AppError, AppResult, Args, write_pretty_json_file, write_retest_horizon_plan_to_s3,
    write_retest_horizon_status_to_s3,
};
use super::kind::RetestRefreshCheckpointKind;

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
