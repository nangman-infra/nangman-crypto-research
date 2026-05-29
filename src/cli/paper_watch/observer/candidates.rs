use super::super::super::*;

pub(super) async fn load_paper_watch_observer_candidates(
    args: &Args,
) -> AppResult<Vec<crate::model::PaperWatchCandidate>> {
    if let Some(path) = args.paper_watch_candidate_file.as_deref() {
        return read_paper_watch_candidates(path);
    }
    let Some(bucket) = args.paper_watch_candidate_s3_bucket.as_deref() else {
        return Err(AppError::config(
            "--run-paper-watch-observer requires --paper-watch-candidate-s3-bucket",
        ));
    };
    let keys = discover_paper_watch_candidate_keys_from_s3(
        bucket,
        &args.paper_watch_candidate_s3_prefix,
        args.paper_watch_observer_read_limit,
        args.paper_watch_observer_scan_limit,
    )
    .await?;
    let mut by_id = BTreeMap::new();
    for key in keys {
        for candidate in read_paper_watch_candidates_from_s3(bucket, &key).await? {
            by_id
                .entry(candidate.paper_watch_candidate_id.clone())
                .or_insert(candidate);
        }
    }
    Ok(by_id.into_values().collect())
}
