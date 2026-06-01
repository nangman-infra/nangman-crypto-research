use super::*;

mod discovery;
mod family;
mod selected_artifact;

use discovery::discover_keys;
use family::MarketArtifactFamily;
use selected_artifact::key_from_selected_artifact;

pub(in crate::cli) async fn market_feature_delta_s3_keys(
    args: &Args,
    bundles: &[IntelCandidateEvidenceBundle],
) -> AppResult<Vec<String>> {
    market_s3_keys(args, bundles, MarketArtifactFamily::FeatureDelta).await
}

pub(in crate::cli) async fn market_regime_context_s3_keys(
    args: &Args,
    bundles: &[IntelCandidateEvidenceBundle],
) -> AppResult<Vec<String>> {
    market_s3_keys(args, bundles, MarketArtifactFamily::RegimeContext).await
}

async fn market_s3_keys(
    args: &Args,
    bundles: &[IntelCandidateEvidenceBundle],
    family: MarketArtifactFamily,
) -> AppResult<Vec<String>> {
    let mut keys = BTreeSet::new();
    for key in family.manual_keys(args) {
        insert_normalized_s3_key(&mut keys, key);
    }
    for bundle in bundles {
        for artifact in &bundle.selected_market_artifacts {
            if let Some(key) = key_from_selected_artifact(family, artifact) {
                insert_normalized_s3_key(&mut keys, &key);
            }
        }
        if let Some(key) = bundle
            .data_quality_summary
            .market_data_quality_summary_key
            .as_deref()
            && let Some(run_id) = market_l1_run_id_from_key(key)
        {
            keys.insert(family.key_from_run_id(&run_id));
        }
    }
    keys.extend(discover_keys(family, args, bundles).await?);
    Ok(keys.into_iter().collect())
}
