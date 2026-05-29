use crate::error::{AppError, AppResult};
use crate::model::PaperWatchLiveMark;
use crate::storage::client::s3_client;
use crate::storage::objects::{put_jsonl_object, put_object_json};
use crate::storage::partition::{normalize_prefix, partition};
use serde::Serialize;

pub async fn write_paper_watch_live_marks_to_s3(
    bucket: &str,
    prefix: &str,
    marks: &[PaperWatchLiveMark],
    output_partition_at_ms: i64,
) -> AppResult<Vec<String>> {
    if marks.is_empty() {
        return Ok(Vec::new());
    }
    if bucket.trim().is_empty() {
        return Err(AppError::config(
            "paper watch live mark output S3 bucket must not be empty",
        ));
    }
    let client = s3_client().await?;
    let dt = partition(output_partition_at_ms)?;
    let prefix = normalize_prefix(if prefix.trim().is_empty() {
        "paper-watch-live-mark/schema=paper_watch_live_mark_v1"
    } else {
        prefix
    });
    if !prefix.starts_with("paper-watch-live-mark/") {
        return Err(AppError::config(
            "paper watch live mark S3 prefix must start with paper-watch-live-mark/",
        ));
    }
    let key = format!(
        "{prefix}dt={}/hour={:02}/run_id={}/part-000001.jsonl",
        dt.date, dt.hour, output_partition_at_ms
    );
    put_jsonl_object(&client, bucket, &key, marks).await?;
    Ok(vec![format!("s3://{bucket}/{key}")])
}

pub async fn write_paper_watch_observer_snapshot_to_s3<T: Serialize>(
    bucket: &str,
    prefix: &str,
    snapshot: &T,
    output_partition_at_ms: i64,
) -> AppResult<String> {
    if bucket.trim().is_empty() {
        return Err(AppError::config(
            "paper-watch observer snapshot output S3 bucket must not be empty",
        ));
    }
    let client = s3_client().await?;
    let dt = partition(output_partition_at_ms)?;
    let prefix = normalize_prefix(if prefix.trim().is_empty() {
        "paper-watch-observer-state/schema=paper_watch_observer_snapshot_v1"
    } else {
        prefix
    });
    if !prefix.starts_with("paper-watch-observer-state/") {
        return Err(AppError::config(
            "paper-watch observer S3 prefix must start with paper-watch-observer-state/",
        ));
    }
    let key = format!(
        "{prefix}dt={}/hour={:02}/run_id={}/state.json",
        dt.date, dt.hour, output_partition_at_ms
    );
    put_object_json(&client, bucket, &key, snapshot).await?;
    Ok(format!("s3://{bucket}/{key}"))
}
