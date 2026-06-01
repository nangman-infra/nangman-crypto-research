use super::*;

pub(super) async fn discover_keys(
    family: MarketArtifactFamily,
    args: &Args,
    bundles: &[IntelCandidateEvidenceBundle],
) -> AppResult<Vec<String>> {
    let starts = market_l1_replay_window_starts(bundles, args.now_ms.unwrap_or_else(now_ms));
    let discovered = match family {
        MarketArtifactFamily::FeatureDelta => {
            discover_latest_market_feature_delta_keys_from_s3(market_l1_s3_bucket(args), &starts)
                .await?
        }
        MarketArtifactFamily::RegimeContext => {
            discover_latest_market_regime_context_keys_from_s3(market_l1_s3_bucket(args), &starts)
                .await?
        }
    };
    Ok(discovered
        .into_iter()
        .filter_map(|key| normalize_s3_key(&key))
        .collect())
}
