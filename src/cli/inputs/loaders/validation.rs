use super::super::*;

pub(in crate::cli) async fn load_oss_adapter_runs(
    args: &Args,
    manifest: Option<&ResearchInputManifest>,
) -> AppResult<Vec<OssAdapterRun>> {
    let mut runs = Vec::new();
    for path in &args.oss_adapter_run_files {
        append_unique_oss_adapter_runs(&mut runs, read_oss_adapter_runs(path)?);
    }
    if let Some(manifest) = manifest {
        for artifact_ref in &manifest.oss_adapter_run_refs {
            append_unique_oss_adapter_runs(
                &mut runs,
                read_oss_adapter_runs_from_ref(artifact_ref).await?,
            );
        }
    }
    if !args.oss_adapter_run_s3_keys.is_empty() {
        let bucket = args
            .oss_adapter_run_s3_bucket
            .as_deref()
            .ok_or_else(|| AppError::config("RESEARCH_OSS_ADAPTER_RUN_S3_BUCKET is required"))?;
        append_unique_oss_adapter_runs(
            &mut runs,
            read_oss_adapter_runs_from_s3(bucket, &args.oss_adapter_run_s3_keys).await?,
        );
    }
    Ok(runs)
}

pub(in crate::cli) async fn load_shadow_validation_runs(
    args: &Args,
    manifest: Option<&ResearchInputManifest>,
) -> AppResult<Vec<ShadowValidationRun>> {
    let mut runs = Vec::new();
    for path in &args.shadow_validation_run_files {
        append_unique_shadow_validation_runs(&mut runs, read_shadow_validation_runs(path)?);
    }
    if let Some(manifest) = manifest {
        for artifact_ref in &manifest.shadow_validation_run_refs {
            append_unique_shadow_validation_runs(
                &mut runs,
                read_shadow_validation_runs_from_ref(artifact_ref).await?,
            );
        }
    }
    if !args.shadow_validation_run_s3_keys.is_empty() {
        let bucket = args
            .shadow_validation_run_s3_bucket
            .as_deref()
            .ok_or_else(|| {
                AppError::config("RESEARCH_SHADOW_VALIDATION_RUN_S3_BUCKET is required")
            })?;
        append_unique_shadow_validation_runs(
            &mut runs,
            read_shadow_validation_runs_from_s3(bucket, &args.shadow_validation_run_s3_keys)
                .await?,
        );
    }
    Ok(runs)
}

pub(in crate::cli) fn validate_oss_adapter_runs(runs: &[OssAdapterRun]) -> AppResult<()> {
    for run in runs {
        if run.schema_version != OSS_ADAPTER_RUN_SCHEMA_VERSION {
            return Err(AppError::validation(format!(
                "oss adapter run schema_version must be {OSS_ADAPTER_RUN_SCHEMA_VERSION}; got {}",
                run.schema_version
            )));
        }
        if !run.lookahead_check_result.eq_ignore_ascii_case("passed") {
            return Err(AppError::validation(format!(
                "oss adapter run {} failed lookahead check: {}",
                run.oss_adapter_run_id, run.lookahead_check_result
            )));
        }
        if !run
            .holding_horizon_check_result
            .eq_ignore_ascii_case("passed")
        {
            return Err(AppError::validation(format!(
                "oss adapter run {} failed holding horizon check: {}",
                run.oss_adapter_run_id, run.holding_horizon_check_result
            )));
        }
    }
    Ok(())
}
