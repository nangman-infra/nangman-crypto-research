use crate::cli::Args;
use crate::error::{AppError, AppResult};

pub(super) fn validate_paper_watch_live_cycle_output(args: &Args) -> AppResult<()> {
    validate_exclusive_output_targets(args)
}

pub(super) fn validate_observer_output(args: &Args) -> AppResult<()> {
    validate_exclusive_output_targets(args)?;
    validate_required_output_target(
        args,
        "--run-paper-watch-observer requires --output-dir or --output-s3-bucket",
    )?;
    if let Some(prefix) = args.output_s3_prefix.as_deref()
        && !prefix.starts_with("paper-watch-observer-state/")
    {
        return Err(AppError::config(
            "--output-s3-prefix must start with paper-watch-observer-state/ in observer mode",
        ));
    }
    Ok(())
}

fn validate_exclusive_output_targets(args: &Args) -> AppResult<()> {
    if args.output_dir.is_some() && args.output_s3_bucket.is_some() {
        return Err(AppError::config(
            "use either --output-dir or --output-s3-bucket, not both",
        ));
    }
    Ok(())
}

fn validate_required_output_target(args: &Args, message: &'static str) -> AppResult<()> {
    if args.output_dir.is_none() && args.output_s3_bucket.is_none() {
        return Err(AppError::config(message));
    }
    Ok(())
}
