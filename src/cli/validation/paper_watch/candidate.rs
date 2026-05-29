use crate::cli::Args;
use crate::error::{AppError, AppResult};

pub(super) fn validate_paper_watch_candidate_input(args: &Args) -> AppResult<()> {
    if args.paper_watch_candidate_file.is_some()
        && (args.paper_watch_candidate_s3_bucket.is_some()
            || args.paper_watch_candidate_s3_key.is_some())
    {
        return Err(AppError::config(
            "use either --paper-watch-candidate-file or --paper-watch-candidate-s3-bucket/--paper-watch-candidate-s3-key, not both",
        ));
    }
    if args.paper_watch_candidate_s3_bucket.is_some() != args.paper_watch_candidate_s3_key.is_some()
    {
        return Err(AppError::config(
            "--paper-watch-candidate-s3-bucket and --paper-watch-candidate-s3-key must be set together",
        ));
    }
    if args.paper_watch_candidate_file.is_none() && args.paper_watch_candidate_s3_key.is_none() {
        return Err(AppError::config(
            "--run-paper-watch-live-cycle requires paper watch candidate input",
        ));
    }
    if let Some(path) = args.paper_watch_candidate_file.as_deref()
        && !path.is_absolute()
    {
        return Err(AppError::config(
            "RESEARCH_PAPER_WATCH_CANDIDATE_FILE must be an absolute path",
        ));
    }
    Ok(())
}
