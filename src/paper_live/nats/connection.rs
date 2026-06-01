use super::*;
use crate::error::AppError;
use async_nats::jetstream;
use async_nats::jetstream::stream::Stream;

pub(super) async fn market_live_stream(config: &MarketLiveNatsConfig) -> AppResult<Stream> {
    let client = async_nats::connect(&config.url)
        .await
        .map_err(|error| AppError::nats(format!("connect {}: {error}", config.url)))?;
    let jetstream = jetstream::new(client);
    jetstream
        .get_stream(&config.stream)
        .await
        .map_err(|error| AppError::nats(format!("get stream {}: {error}", config.stream)))
}
