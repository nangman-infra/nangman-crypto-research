use super::*;

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
            if let Some(key) = family.key_from_selected_artifact(artifact) {
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
    keys.extend(family.discover_keys(args, bundles).await?);
    Ok(keys.into_iter().collect())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MarketArtifactFamily {
    FeatureDelta,
    RegimeContext,
}

impl MarketArtifactFamily {
    fn manual_keys(self, args: &Args) -> &[String] {
        match self {
            Self::FeatureDelta => &args.market_feature_delta_s3_keys,
            Self::RegimeContext => &args.market_regime_context_s3_keys,
        }
    }

    fn key_from_selected_artifact(self, artifact: &SelectedMarketArtifactTrace) -> Option<String> {
        match self {
            Self::FeatureDelta => feature_delta_key_from_selected_artifact(artifact),
            Self::RegimeContext => (artifact.artifact_type == MARKET_REGIME_CONTEXT_ARTIFACT_TYPE)
                .then(|| artifact.artifact_key.clone())
                .flatten(),
        }
    }

    fn key_from_run_id(self, run_id: &str) -> String {
        match self {
            Self::FeatureDelta => format!("market_feature_delta/run_id={run_id}/delta.json"),
            Self::RegimeContext => {
                format!("market_regime_context/run_id={run_id}/context.json")
            }
        }
    }

    async fn discover_keys(
        self,
        args: &Args,
        bundles: &[IntelCandidateEvidenceBundle],
    ) -> AppResult<Vec<String>> {
        let starts = market_l1_replay_window_starts(bundles, args.now_ms.unwrap_or_else(now_ms));
        let discovered = match self {
            Self::FeatureDelta => {
                discover_latest_market_feature_delta_keys_from_s3(
                    market_l1_s3_bucket(args),
                    &starts,
                )
                .await?
            }
            Self::RegimeContext => {
                discover_latest_market_regime_context_keys_from_s3(
                    market_l1_s3_bucket(args),
                    &starts,
                )
                .await?
            }
        };
        Ok(discovered
            .into_iter()
            .filter_map(|key| normalize_s3_key(&key))
            .collect())
    }
}

pub(in crate::cli) fn feature_delta_key_from_selected_artifact(
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
