use super::*;

pub(super) fn validate_default_research_io_args(args: &Args) -> AppResult<()> {
    validate_research_input_sources(args)?;
    validate_research_output_target(args)?;
    validate_market_context_sources(args)?;
    validate_historical_artifact_sources(args)
}

fn validate_research_input_sources(args: &Args) -> AppResult<()> {
    if args.input_bundle_file.is_some()
        && (args.input_bundle_s3_bucket.is_some() || args.input_bundle_s3_key.is_some())
    {
        return Err(AppError::config(
            "use either --input-bundle-file or --input-bundle-s3-bucket/--input-bundle-s3-key, not both",
        ));
    }
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
    if args.input_bundle_file.is_none()
        && args.input_manifest_file.is_none()
        && args.input_manifest_s3_key.is_none()
        && (args.input_bundle_s3_bucket.is_none() || args.input_bundle_s3_key.is_none())
    {
        return Err(AppError::config(
            "--input-bundle-file or --input-manifest-file is required unless S3 input environment is set",
        ));
    }
    Ok(())
}

fn validate_research_output_target(args: &Args) -> AppResult<()> {
    if args.output_dir.is_some() && args.output_s3_bucket.is_some() {
        return Err(AppError::config(
            "use either --output-dir or --output-s3-bucket, not both",
        ));
    }
    Ok(())
}

fn validate_market_context_sources(args: &Args) -> AppResult<()> {
    if args.market_feature_delta_file.is_some() && !args.market_feature_delta_s3_keys.is_empty() {
        return Err(AppError::config(
            "use either --market-feature-delta-file or --market-feature-delta-s3-key, not both",
        ));
    }
    if args.market_regime_context_file.is_some() && !args.market_regime_context_s3_keys.is_empty() {
        return Err(AppError::config(
            "use either --market-regime-context-file or --market-regime-context-s3-key, not both",
        ));
    }
    Ok(())
}

fn validate_historical_artifact_sources(args: &Args) -> AppResult<()> {
    if !args.historical_replay_run_s3_keys.is_empty()
        && args.historical_replay_run_s3_bucket.is_none()
    {
        return Err(AppError::config(
            "--historical-replay-run-s3-bucket is required when --historical-replay-run-s3-key is set",
        ));
    }
    if !args.historical_replay_run_index_s3_keys.is_empty()
        && args.historical_replay_run_index_s3_bucket.is_none()
    {
        return Err(AppError::config(
            "--historical-replay-run-index-s3-bucket is required when --historical-replay-run-index-s3-key is set",
        ));
    }
    if !args.oss_adapter_run_s3_keys.is_empty() && args.oss_adapter_run_s3_bucket.is_none() {
        return Err(AppError::config(
            "--oss-adapter-run-s3-bucket is required when --oss-adapter-run-s3-key is set",
        ));
    }
    if !args.shadow_validation_run_s3_keys.is_empty()
        && args.shadow_validation_run_s3_bucket.is_none()
    {
        return Err(AppError::config(
            "--shadow-validation-run-s3-bucket is required when --shadow-validation-run-s3-key is set",
        ));
    }
    Ok(())
}
