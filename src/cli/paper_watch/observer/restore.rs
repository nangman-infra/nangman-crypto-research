use super::super::super::*;

pub(super) async fn restore_paper_watch_observer_state(
    args: &Args,
    state: &mut PaperWatchObserverState,
) -> AppResult<usize> {
    let Some(bucket) = args.output_s3_bucket.as_deref() else {
        return Ok(0);
    };
    let restore_prefix = hourly_partitioned_prefix(
        &args.paper_watch_live_mark_s3_prefix,
        args.now_ms.unwrap_or_else(now_ms),
    )?;
    let keys = discover_paper_watch_live_mark_keys_from_s3(
        bucket,
        &restore_prefix,
        args.paper_watch_live_mark_read_limit,
        args.paper_watch_live_mark_scan_limit,
    )
    .await?;
    if keys.is_empty() {
        return Ok(0);
    }
    let marks = read_paper_watch_live_marks_from_s3(bucket, &keys).await?;
    let restored_count = marks.len();
    state.restore_marks(&marks);
    Ok(restored_count)
}
