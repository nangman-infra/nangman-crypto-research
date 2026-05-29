use super::super::super::*;
use serde::Serialize;

pub(in crate::cli) async fn write_paper_watch_observer_live_marks(
    args: &Args,
    marks: &[crate::model::PaperWatchLiveMark],
    output_partition_at_ms: i64,
) -> AppResult<Vec<String>> {
    if marks.is_empty() {
        return Ok(Vec::new());
    }
    if let Some(output_dir) = args.output_dir.as_deref() {
        return write_paper_watch_live_marks(output_dir, marks, output_partition_at_ms).map(
            |paths| {
                paths
                    .into_iter()
                    .map(|path| path.display().to_string())
                    .collect()
            },
        );
    }
    let Some(bucket) = args.output_s3_bucket.as_deref() else {
        return Ok(Vec::new());
    };
    write_paper_watch_live_marks_to_s3(
        bucket,
        &args.paper_watch_live_mark_s3_prefix,
        marks,
        output_partition_at_ms,
    )
    .await
}

pub(in crate::cli) async fn write_paper_watch_observer_snapshot<T: Serialize>(
    args: &Args,
    snapshot: &T,
    output_partition_at_ms: i64,
) -> AppResult<String> {
    if let Some(output_dir) = args.output_dir.as_deref() {
        let path = output_dir.join(format!(
            "paper-watch-observer-state/schema={}/run_id={}/state.json",
            PAPER_WATCH_OBSERVER_SNAPSHOT_SCHEMA_VERSION, output_partition_at_ms
        ));
        return write_pretty_json_file(&path, snapshot).map(|path| path.display().to_string());
    }
    let Some(bucket) = args.output_s3_bucket.as_deref() else {
        return Err(AppError::config(
            "--run-paper-watch-observer requires --output-dir or --output-s3-bucket",
        ));
    };
    write_paper_watch_observer_snapshot_to_s3(
        bucket,
        args.output_s3_prefix
            .as_deref()
            .unwrap_or(DEFAULT_PAPER_WATCH_OBSERVER_OUTPUT_PREFIX),
        snapshot,
        output_partition_at_ms,
    )
    .await
}
