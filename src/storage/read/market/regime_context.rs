use super::super::super::{
    AppResult, MarketRegimeContext, discover_latest_market_l1_keys_from_s3, get_object_bytes,
    is_missing_market_artifact, read_market_regime_contexts_from_bytes, s3_client,
};

pub async fn read_market_regime_contexts_from_s3(
    bucket: &str,
    keys: &[String],
) -> AppResult<Vec<MarketRegimeContext>> {
    let client = s3_client().await?;
    let mut contexts = Vec::new();
    for key in keys {
        let bytes = match get_object_bytes(&client, bucket, key).await {
            Ok(bytes) => bytes,
            Err(error) if is_missing_market_artifact(&error) => continue,
            Err(error) => return Err(error),
        };
        contexts.extend(read_market_regime_contexts_from_bytes(
            &format!("s3://{bucket}/{key}"),
            bytes.as_ref(),
        )?);
    }
    Ok(contexts)
}

pub async fn discover_latest_market_regime_context_keys_from_s3(
    bucket: &str,
    window_starts_ms: &[i64],
) -> AppResult<Vec<String>> {
    discover_latest_market_l1_keys_from_s3(
        bucket,
        window_starts_ms,
        "market_regime_context",
        "/context.json",
        "market_regime_context_key",
    )
    .await
}
