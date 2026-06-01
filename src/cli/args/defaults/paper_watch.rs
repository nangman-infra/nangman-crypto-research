use super::*;

pub(super) fn apply_paper_watch_env(args: &mut Args) -> AppResult<()> {
    args.paper_watch_candidate_file =
        env_string("RESEARCH_PAPER_WATCH_CANDIDATE_FILE").map(PathBuf::from);
    args.paper_watch_candidate_s3_bucket = env_string("RESEARCH_PAPER_WATCH_CANDIDATE_S3_BUCKET");
    args.paper_watch_candidate_s3_key = env_string("RESEARCH_PAPER_WATCH_CANDIDATE_S3_KEY");
    args.paper_watch_candidate_s3_prefix = env_string("RESEARCH_PAPER_WATCH_CANDIDATE_S3_PREFIX")
        .unwrap_or_else(|| DEFAULT_PAPER_WATCH_CANDIDATE_PREFIX.to_owned());
    args.paper_watch_observer_read_limit = env_usize(
        "RESEARCH_PAPER_WATCH_OBSERVER_READ_LIMIT",
        DEFAULT_PAPER_WATCH_OBSERVER_READ_LIMIT,
    )?;
    args.paper_watch_observer_scan_limit = env_usize(
        "RESEARCH_PAPER_WATCH_OBSERVER_SCAN_LIMIT",
        DEFAULT_PAPER_WATCH_OBSERVER_SCAN_LIMIT,
    )?;
    args.paper_watch_observer_poll_secs = env_u64(
        "RESEARCH_PAPER_WATCH_OBSERVER_POLL_SECS",
        DEFAULT_PAPER_WATCH_OBSERVER_POLL_SECS,
    )?;
    args.paper_watch_observer_max_iterations =
        env_usize_allow_zero("RESEARCH_PAPER_WATCH_OBSERVER_MAX_ITERATIONS", 0)?;
    args.paper_watch_live_mark_s3_prefix = env_string("RESEARCH_PAPER_WATCH_LIVE_MARK_S3_PREFIX")
        .unwrap_or_else(|| DEFAULT_PAPER_WATCH_LIVE_MARK_PREFIX.to_owned());
    args.paper_watch_live_mark_read_limit = env_usize(
        "RESEARCH_PAPER_WATCH_LIVE_MARK_READ_LIMIT",
        DEFAULT_PAPER_WATCH_OBSERVER_READ_LIMIT,
    )?;
    args.paper_watch_live_mark_scan_limit = env_usize(
        "RESEARCH_PAPER_WATCH_LIVE_MARK_SCAN_LIMIT",
        DEFAULT_PAPER_WATCH_OBSERVER_SCAN_LIMIT,
    )?;
    Ok(())
}
