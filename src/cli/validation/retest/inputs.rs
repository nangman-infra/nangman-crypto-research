use crate::cli::Args;
use crate::error::{AppError, AppResult};
use crate::path_validation::validate_config_absolute_path;

pub(in crate::cli) fn validate_retest_horizon_plan_input_args(args: &Args) -> AppResult<()> {
    if args.retest_horizon_plan_file.is_some()
        && (args.retest_horizon_plan_s3_bucket.is_some()
            || args.retest_horizon_plan_s3_key.is_some())
    {
        return Err(AppError::config(
            "use either --retest-horizon-plan-file or --retest-horizon-plan-s3-bucket/--retest-horizon-plan-s3-key, not both",
        ));
    }
    if args.retest_horizon_plan_s3_bucket.is_some() != args.retest_horizon_plan_s3_key.is_some() {
        return Err(AppError::config(
            "--retest-horizon-plan-s3-bucket and --retest-horizon-plan-s3-key must be set together",
        ));
    }
    if let Some(path) = args.retest_horizon_plan_file.as_deref() {
        validate_config_absolute_path(path, "RESEARCH_RETEST_HORIZON_PLAN_FILE")?;
    }
    if let Some(path) = args.retest_driver_summary_file.as_deref() {
        validate_config_absolute_path(path, "RESEARCH_RETEST_DRIVER_SUMMARY_FILE")?;
    }
    Ok(())
}

pub(in crate::cli) fn validate_research_report_input_args(args: &Args) -> AppResult<()> {
    if args.research_report_file.is_some()
        && (args.research_report_s3_bucket.is_some() || args.research_report_s3_key.is_some())
    {
        return Err(AppError::config(
            "use either --research-report-file or --research-report-s3-bucket/--research-report-s3-key, not both",
        ));
    }
    if args.research_report_s3_bucket.is_some() != args.research_report_s3_key.is_some() {
        return Err(AppError::config(
            "--research-report-s3-bucket and --research-report-s3-key must be set together",
        ));
    }
    if let Some(path) = args.research_report_file.as_deref() {
        validate_config_absolute_path(path, "RESEARCH_REPORT_FILE")?;
    }
    Ok(())
}

pub(in crate::cli) fn validate_retest_horizon_status_input_args(args: &Args) -> AppResult<()> {
    if args.retest_horizon_status_file.is_some()
        && (args.retest_horizon_status_s3_bucket.is_some()
            || args.retest_horizon_status_s3_key.is_some())
    {
        return Err(AppError::config(
            "use either --retest-horizon-status-file or --retest-horizon-status-s3-bucket/--retest-horizon-status-s3-key, not both",
        ));
    }
    if args.retest_horizon_status_s3_bucket.is_some() != args.retest_horizon_status_s3_key.is_some()
    {
        return Err(AppError::config(
            "--retest-horizon-status-s3-bucket and --retest-horizon-status-s3-key must be set together",
        ));
    }
    if let Some(path) = args.retest_horizon_status_file.as_deref() {
        validate_config_absolute_path(path, "RESEARCH_HORIZON_STATUS_FILE")?;
    }
    Ok(())
}
