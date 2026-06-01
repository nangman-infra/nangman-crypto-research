use super::*;

pub(super) fn key_from_selected_artifact(
    family: MarketArtifactFamily,
    artifact: &SelectedMarketArtifactTrace,
) -> Option<String> {
    match family {
        MarketArtifactFamily::FeatureDelta => feature_delta_key_from_selected_artifact(artifact),
        MarketArtifactFamily::RegimeContext => (artifact.artifact_type
            == MARKET_REGIME_CONTEXT_ARTIFACT_TYPE)
            .then(|| artifact.artifact_key.clone())
            .flatten(),
    }
}

fn feature_delta_key_from_selected_artifact(
    artifact: &SelectedMarketArtifactTrace,
) -> Option<String> {
    if artifact.artifact_type == MARKET_FEATURE_DELTA_ARTIFACT_TYPE {
        return artifact.artifact_key.clone();
    }
    if artifact.artifact_type != MARKET_FEATURE_DELTA_SUMMARY_ARTIFACT_TYPE {
        return None;
    }
    artifact
        .l1_run_id
        .clone()
        .or_else(|| {
            artifact
                .artifact_key
                .as_deref()
                .and_then(market_l1_run_id_from_key)
        })
        .map(|run_id| format!("market_feature_delta/run_id={run_id}/delta.json"))
}
