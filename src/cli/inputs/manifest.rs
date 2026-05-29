use super::*;

pub(in crate::cli) async fn load_input_manifest(
    args: &Args,
) -> AppResult<Option<ResearchInputManifest>> {
    if let Some(path) = args.input_manifest_file.as_deref() {
        return read_research_input_manifest(path).map(Some);
    }
    match (
        args.input_manifest_s3_bucket.as_deref(),
        args.input_manifest_s3_key.as_deref(),
    ) {
        (Some(bucket), Some(key)) => read_research_input_manifest_from_s3(bucket, key)
            .await
            .map(Some),
        _ => Ok(None),
    }
}

pub(in crate::cli) fn validate_input_manifest(
    manifest: Option<&ResearchInputManifest>,
) -> AppResult<()> {
    let Some(manifest) = manifest else {
        return Ok(());
    };
    if manifest.schema_version != RESEARCH_INPUT_MANIFEST_SCHEMA_VERSION {
        return Err(AppError::validation(format!(
            "research input manifest schema_version must be {RESEARCH_INPUT_MANIFEST_SCHEMA_VERSION}; got {}",
            manifest.schema_version
        )));
    }
    for artifact_ref in all_manifest_refs(manifest) {
        validate_artifact_ref(artifact_ref)?;
    }
    Ok(())
}

pub(in crate::cli) fn validate_manifest_budget(
    manifest: Option<&ResearchInputManifest>,
    budget: &ResearchRuntimeBudgetPolicy,
) -> AppResult<()> {
    let Some(manifest) = manifest else {
        return Ok(());
    };
    enforce_budget(
        "candidate_bundle_ref_count",
        manifest.candidate_bundle_refs.len(),
        budget.max_candidate_bundle_count,
    )?;
    enforce_budget(
        "market_artifact_ref_count",
        manifest.market_feature_delta_refs.len() + manifest.market_regime_context_refs.len(),
        budget.max_market_artifact_ref_count,
    )?;
    enforce_budget(
        "shadow_validation_run_ref_count",
        manifest.shadow_validation_run_refs.len(),
        budget.max_shadow_validation_run_ref_count,
    )?;
    enforce_budget(
        "hypothesis_harness_result_ref_count",
        manifest.hypothesis_harness_result_refs.len(),
        budget.max_hypothesis_harness_result_ref_count,
    )?;
    enforce_budget(
        "oss_adapter_run_ref_count",
        manifest.oss_adapter_run_refs.len(),
        budget.max_oss_adapter_run_ref_count,
    )?;
    enforce_budget(
        "historical_replay_run_ref_count",
        manifest.historical_replay_run_refs.len() + manifest.historical_replay_run_index_refs.len(),
        budget.max_historical_replay_run_ref_count,
    )?;
    Ok(())
}

pub(in crate::cli) fn enforce_budget(name: &str, actual: usize, maximum: usize) -> AppResult<()> {
    if maximum == 0 {
        return Err(AppError::config(format!(
            "runtime_budget_policy.{name} maximum must be greater than zero"
        )));
    }
    if actual > maximum {
        return Err(AppError::validation(format!(
            "runtime budget exceeded for {name}: actual={actual}, max={maximum}"
        )));
    }
    Ok(())
}
