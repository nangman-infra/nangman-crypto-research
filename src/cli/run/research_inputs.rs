use super::*;

pub(super) struct ResearchInputs {
    pub(super) manifest: Option<ResearchInputManifest>,
    pub(super) budget: ResearchRuntimeBudgetPolicy,
    pub(super) bundles: Vec<IntelCandidateEvidenceBundle>,
    pub(super) market_deltas: Vec<MarketFeatureDelta>,
    pub(super) regime_contexts: Vec<MarketRegimeContext>,
    pub(super) oss_adapter_runs: Vec<OssAdapterRun>,
    pub(super) shadow_validation_runs: Vec<ShadowValidationRun>,
}

pub(super) async fn load_research_inputs(args: &Args) -> AppResult<ResearchInputs> {
    let manifest = load_input_manifest(args).await?;
    validate_input_manifest(manifest.as_ref())?;
    let budget = manifest
        .as_ref()
        .map(|manifest| manifest.runtime_budget_policy.clone())
        .unwrap_or_default();
    validate_manifest_budget(manifest.as_ref(), &budget)?;

    let bundles = read_input_bundles(args, manifest.as_ref()).await?;
    if bundles.is_empty() {
        return Err(AppError::validation("input bundle file must not be empty"));
    }
    enforce_budget(
        "candidate_bundle_count",
        bundles.len(),
        budget.max_candidate_bundle_count,
    )?;

    let market_deltas = load_market_deltas(
        args,
        &bundles,
        manifest.as_ref(),
        budget.max_market_artifact_ref_count,
    )
    .await?;
    let regime_contexts = load_regime_contexts(
        args,
        &bundles,
        manifest.as_ref(),
        budget.max_market_artifact_ref_count,
    )
    .await?;
    let oss_adapter_runs = load_oss_adapter_runs(args, manifest.as_ref()).await?;
    let shadow_validation_runs = load_shadow_validation_runs(args, manifest.as_ref()).await?;
    validate_oss_adapter_runs(&oss_adapter_runs)?;

    Ok(ResearchInputs {
        manifest,
        budget,
        bundles,
        market_deltas,
        regime_contexts,
        oss_adapter_runs,
        shadow_validation_runs,
    })
}
