use super::super::super::{
    AppResult, MarketFeatureDelta, discover_latest_market_l1_keys_from_s3, get_object_bytes,
    is_missing_market_artifact, read_market_feature_deltas_matching_symbols_from_bytes, s3_client,
};
use std::collections::BTreeSet;

pub async fn read_market_feature_deltas_from_s3(
    bucket: &str,
    keys: &[String],
    symbols: &BTreeSet<String>,
) -> AppResult<Vec<MarketFeatureDelta>> {
    let client = s3_client().await?;
    let mut deltas = Vec::new();
    for key in keys {
        let bytes = match get_object_bytes(&client, bucket, key).await {
            Ok(bytes) => bytes,
            Err(error) if is_missing_market_artifact(&error) => continue,
            Err(error) => return Err(error),
        };
        deltas.extend(read_market_feature_deltas_matching_symbols_from_bytes(
            &format!("s3://{bucket}/{key}"),
            bytes.as_ref(),
            symbols,
        )?);
    }
    Ok(deltas)
}

pub async fn discover_latest_market_feature_delta_keys_from_s3(
    bucket: &str,
    window_starts_ms: &[i64],
) -> AppResult<Vec<String>> {
    discover_latest_market_l1_keys_from_s3(
        bucket,
        window_starts_ms,
        "market_feature_delta",
        "/delta.json",
        "market_feature_delta_key",
    )
    .await
}
