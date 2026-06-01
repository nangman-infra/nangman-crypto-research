use super::*;
use crate::error::AppError;
use crate::model::MarketLiveTick;
use async_nats::jetstream::consumer::PullConsumer;
use futures_util::StreamExt;
use std::time::Duration;
use tokio::time::timeout;

pub(super) async fn read_ticks_from_consumer(
    consumer: PullConsumer,
    config: &MarketLiveNatsConfig,
) -> AppResult<Vec<MarketLiveTick>> {
    let mut ticks = Vec::new();
    let fetch_timeout = Duration::from_secs(6);
    let ack_timeout = Duration::from_secs(config.ack_wait_secs.clamp(1, 5));
    while ticks.len() < config.max_messages {
        let remaining = config.max_messages - ticks.len();
        let batch_size = config.batch_size.min(remaining).max(1);
        let mut messages = timeout(
            fetch_timeout,
            consumer
                .fetch()
                .max_messages(batch_size)
                .expires(Duration::from_secs(5))
                .messages(),
        )
        .await
        .map_err(|_| AppError::nats("fetch market live messages timed out".to_owned()))?
        .map_err(|error| AppError::nats(format!("fetch market live messages: {error}")))?;
        let Some(message) = timeout(fetch_timeout, messages.next())
            .await
            .map_err(|_| AppError::nats("read market live message timed out".to_owned()))?
        else {
            break;
        };
        let message = message
            .map_err(|error| AppError::nats(format!("read market live message: {error}")))?;
        let tick: MarketLiveTick = serde_json::from_slice(&message.payload)?;
        tick::validate_tick(&tick)?;
        timeout(ack_timeout, message.ack())
            .await
            .map_err(|_| AppError::nats("market live ack timed out".to_owned()))?
            .map_err(|error| AppError::nats(format!("market live ack failed: {error}")))?;
        ticks.push(tick);
    }
    Ok(ticks)
}
