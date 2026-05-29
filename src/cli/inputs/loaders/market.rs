use super::super::*;

pub(in crate::cli) async fn load_market_deltas(
    args: &Args,
    bundles: &[IntelCandidateEvidenceBundle],
    manifest: Option<&ResearchInputManifest>,
    max_market_artifact_ref_count: usize,
) -> AppResult<Vec<MarketFeatureDelta>> {
    let mut deltas = Vec::new();
    if let Some(path) = args.market_feature_delta_file.as_deref() {
        deltas.extend(read_market_feature_deltas(path)?);
    }
    if let Some(manifest) = manifest {
        for artifact_ref in &manifest.market_feature_delta_refs {
            deltas.extend(read_market_feature_deltas_from_ref(artifact_ref).await?);
        }
    }
    if !should_read_market_s3(args) {
        return Ok(deltas);
    }
    let keys = market_feature_delta_s3_keys(args, bundles).await?;
    enforce_budget(
        "market_feature_delta_s3_key_count",
        keys.len(),
        max_market_artifact_ref_count,
    )?;
    if keys.is_empty() {
        return Ok(deltas);
    }
    let symbols = bundle_symbol_filter(bundles);
    deltas.extend(
        read_market_feature_deltas_from_s3(market_l1_s3_bucket(args), &keys, &symbols).await?,
    );
    Ok(deltas)
}

pub(in crate::cli) async fn load_regime_contexts(
    args: &Args,
    bundles: &[IntelCandidateEvidenceBundle],
    manifest: Option<&ResearchInputManifest>,
    max_market_artifact_ref_count: usize,
) -> AppResult<Vec<MarketRegimeContext>> {
    let mut contexts = Vec::new();
    if let Some(path) = args.market_regime_context_file.as_deref() {
        contexts.extend(read_market_regime_contexts(path)?);
    }
    if let Some(manifest) = manifest {
        for artifact_ref in &manifest.market_regime_context_refs {
            contexts.extend(read_market_regime_contexts_from_ref(artifact_ref).await?);
        }
    }
    if !should_read_market_s3(args) {
        return Ok(contexts);
    }
    let keys = market_regime_context_s3_keys(args, bundles).await?;
    enforce_budget(
        "market_regime_context_s3_key_count",
        keys.len(),
        max_market_artifact_ref_count,
    )?;
    if keys.is_empty() {
        return Ok(contexts);
    }
    contexts.extend(read_market_regime_contexts_from_s3(market_l1_s3_bucket(args), &keys).await?);
    Ok(contexts)
}
