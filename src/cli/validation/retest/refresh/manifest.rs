use crate::cli::Args;
use crate::error::{AppError, AppResult};

pub(super) fn validate_retest_refresh_manifest_input(args: &Args) -> AppResult<()> {
    match (
        args.input_manifest_file.is_some(),
        args.input_manifest_s3_bucket.is_some(),
        args.input_manifest_s3_key.is_some(),
    ) {
        (true, false, false) | (false, true, true) => Ok(()),
        (true, _, _) => Err(AppError::config(
            "use either --input-manifest-file or --input-manifest-s3-bucket/--input-manifest-s3-key, not both",
        )),
        (false, true, false) | (false, false, true) => Err(AppError::config(
            "RESEARCH_INPUT_MANIFEST_S3_BUCKET and RESEARCH_INPUT_MANIFEST_S3_KEY must be set together",
        )),
        (false, false, false) => Err(AppError::config(
            "--run-retest-refresh-cycle requires --input-manifest-file or S3 manifest input",
        )),
    }
}
