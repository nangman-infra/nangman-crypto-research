use super::super::*;
use super::sources::{load_market_live_ticks, load_paper_watch_candidates};

pub(in crate::cli) async fn run_paper_watch_live_cycle_mode(args: &Args) -> AppResult<RunSummary> {
    let candidates = load_paper_watch_candidates(args).await?;
    if candidates.is_empty() {
        return Err(AppError::validation(
            "paper watch candidate input must not be empty",
        ));
    }
    let output_partition_at_ms = args.now_ms.unwrap_or_else(now_ms);
    let ticks = load_market_live_ticks(args, &candidates, output_partition_at_ms).await?;
    let marks = build_paper_watch_live_marks(&candidates, &ticks);
    let output_files = if let Some(output_dir) = args.output_dir.as_deref() {
        write_paper_watch_live_marks(output_dir, &marks, output_partition_at_ms)?
            .into_iter()
            .map(|path| path.display().to_string())
            .collect()
    } else if let Some(output_bucket) = args.output_s3_bucket.as_deref() {
        write_paper_watch_live_marks_to_s3(
            output_bucket,
            args.output_s3_prefix.as_deref().unwrap_or(""),
            &marks,
            output_partition_at_ms,
        )
        .await?
    } else {
        println!("{}", serde_json::to_string_pretty(&marks)?);
        Vec::new()
    };
    emit_paper_watch_live_mark_alert_from_env(&marks).await;

    Ok(RunSummary {
        paper_watch_live_marks_created: marks.len(),
        output_files,
        ..RunSummary::default()
    })
}
