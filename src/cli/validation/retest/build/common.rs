use crate::cli::Args;
use crate::error::{AppError, AppResult};

pub(super) fn validate_manifest_input_source(
    args: &Args,
    missing_input_message: &'static str,
) -> AppResult<()> {
    if args.input_manifest_file.is_some()
        && (args.input_manifest_s3_bucket.is_some() || args.input_manifest_s3_key.is_some())
    {
        return Err(AppError::config(
            "use either --input-manifest-file or --input-manifest-s3-bucket/--input-manifest-s3-key, not both",
        ));
    }
    if args.input_manifest_s3_bucket.is_some() != args.input_manifest_s3_key.is_some() {
        return Err(AppError::config(
            "RESEARCH_INPUT_MANIFEST_S3_BUCKET and RESEARCH_INPUT_MANIFEST_S3_KEY must be set together",
        ));
    }
    if args.input_manifest_file.is_none() && args.input_manifest_s3_key.is_none() {
        return Err(AppError::config(missing_input_message));
    }
    Ok(())
}

pub(super) fn validate_output_target(
    output_file_selected: bool,
    output_s3_selected: bool,
    output_dir_selected: bool,
    conflict_message: &'static str,
    output_dir_message: &'static str,
    missing_output_message: &'static str,
) -> AppResult<()> {
    if output_s3_selected && output_file_selected {
        return Err(AppError::config(conflict_message));
    }
    if output_dir_selected {
        return Err(AppError::config(output_dir_message));
    }
    if !output_file_selected && !output_s3_selected {
        return Err(AppError::config(missing_output_message));
    }
    Ok(())
}
