use super::market_live::validate_market_live_url;
use super::mode::validate_paper_watch_observer_mode_is_isolated;
use super::output::validate_observer_output;
use crate::cli::Args;
use crate::error::{AppError, AppResult};
use crate::path_validation::validate_config_absolute_path;

pub(in crate::cli) fn validate_paper_watch_observer_args(args: &Args) -> AppResult<()> {
    validate_paper_watch_observer_mode_is_isolated(args)?;
    validate_observer_candidate_input(args)?;
    validate_observer_prefixes(args)?;
    validate_market_live_url(args)?;
    validate_observer_output(args)?;
    Ok(())
}

fn validate_observer_candidate_input(args: &Args) -> AppResult<()> {
    if args.paper_watch_candidate_file.is_some() && args.paper_watch_candidate_s3_bucket.is_some() {
        return Err(AppError::config(
            "use either --paper-watch-candidate-file or --paper-watch-candidate-s3-bucket, not both",
        ));
    }
    if args.paper_watch_candidate_file.is_none() && args.paper_watch_candidate_s3_bucket.is_none() {
        return Err(AppError::config(
            "--run-paper-watch-observer requires --paper-watch-candidate-s3-bucket or --paper-watch-candidate-file",
        ));
    }
    if args.paper_watch_candidate_s3_key.is_some() {
        return Err(AppError::config(
            "--run-paper-watch-observer uses --paper-watch-candidate-s3-prefix, not --paper-watch-candidate-s3-key",
        ));
    }
    if let Some(path) = args.paper_watch_candidate_file.as_deref() {
        validate_config_absolute_path(path, "RESEARCH_PAPER_WATCH_CANDIDATE_FILE")?;
    }
    Ok(())
}

fn validate_observer_prefixes(args: &Args) -> AppResult<()> {
    if !args
        .paper_watch_candidate_s3_prefix
        .starts_with("paper-watch-candidate/")
    {
        return Err(AppError::config(
            "--paper-watch-candidate-s3-prefix must start with paper-watch-candidate/",
        ));
    }
    if !args
        .paper_watch_live_mark_s3_prefix
        .starts_with("paper-watch-live-mark/")
    {
        return Err(AppError::config(
            "--paper-watch-live-mark-s3-prefix must start with paper-watch-live-mark/",
        ));
    }
    Ok(())
}
