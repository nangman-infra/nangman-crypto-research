mod connection;
mod consumer;
mod reader;
mod tick;

use super::config::{MarketLiveNatsConfig, validate_nats_config};
use crate::error::AppResult;
use crate::model::MarketLiveTick;
use connection::market_live_stream;
use consumer::{delete_market_live_consumer, market_live_consumer};
use reader::read_ticks_from_consumer;

#[cfg(test)]
pub(super) use tick::validate_tick;

pub async fn read_market_live_ticks_from_nats(
    config: &MarketLiveNatsConfig,
) -> AppResult<Vec<MarketLiveTick>> {
    validate_nats_config(config)?;
    let stream = market_live_stream(config).await?;
    let consumer = market_live_consumer(&stream, config).await?;
    let ticks_result = read_ticks_from_consumer(consumer, config).await;
    let delete_result = delete_market_live_consumer(&stream, config).await;
    match (ticks_result, delete_result) {
        (Ok(ticks), Ok(())) => Ok(ticks),
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(error),
    }
}
