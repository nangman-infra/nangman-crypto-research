use super::config::{MarketLiveNatsConfig, deliver_policy, validate_nats_config};
use super::price::valid_mark_price;
use crate::error::{AppError, AppResult};
use crate::model::{MARKET_LIVE_TICK_SCHEMA_VERSION, MarketLiveTick};
use async_nats::jetstream;
use async_nats::jetstream::consumer::AckPolicy;
use async_nats::jetstream::consumer::PullConsumer;
use futures_util::StreamExt;
use std::time::Duration;
use tokio::time::timeout;

pub async fn read_market_live_ticks_from_nats(
    config: &MarketLiveNatsConfig,
) -> AppResult<Vec<MarketLiveTick>> {
    validate_nats_config(config)?;
    let client = async_nats::connect(&config.url)
        .await
        .map_err(|error| AppError::nats(format!("connect {}: {error}", config.url)))?;
    let jetstream = jetstream::new(client);
    let stream = jetstream
        .get_stream(&config.stream)
        .await
        .map_err(|error| AppError::nats(format!("get stream {}: {error}", config.stream)))?;
    let consumer = stream
        .get_or_create_consumer(
            &config.consumer,
            jetstream::consumer::pull::Config {
                durable_name: Some(config.consumer.clone()),
                filter_subject: config.subject.clone(),
                ack_policy: AckPolicy::Explicit,
                ack_wait: Duration::from_secs(config.ack_wait_secs),
                max_ack_pending: config.batch_size as i64,
                deliver_policy: deliver_policy(&config.deliver_policy)?,
                ..Default::default()
            },
        )
        .await
        .map_err(|error| {
            AppError::nats(format!(
                "get/create consumer {} on stream {}: {error}",
                config.consumer, config.stream
            ))
        })?;

    let ticks_result = read_ticks_from_consumer(consumer, config).await;
    let delete_result = if config.delete_consumer_after_read {
        stream.delete_consumer(&config.consumer).await.map(Some)
    } else {
        Ok(None)
    };
    match (ticks_result, delete_result) {
        (Ok(ticks), Ok(_)) => Ok(ticks),
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(AppError::nats(format!(
            "delete market live consumer {} on stream {}: {error}",
            config.consumer, config.stream
        ))),
    }
}

async fn read_ticks_from_consumer(
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
        validate_tick(&tick)?;
        timeout(ack_timeout, message.ack())
            .await
            .map_err(|_| AppError::nats("market live ack timed out".to_owned()))?
            .map_err(|error| AppError::nats(format!("market live ack failed: {error}")))?;
        ticks.push(tick);
    }
    Ok(ticks)
}

pub(super) fn validate_tick(tick: &MarketLiveTick) -> AppResult<()> {
    if tick.schema_version != MARKET_LIVE_TICK_SCHEMA_VERSION {
        return Err(AppError::validation(format!(
            "market live tick schema_version must be {MARKET_LIVE_TICK_SCHEMA_VERSION}; got {}",
            tick.schema_version
        )));
    }
    if tick.event_id.trim().is_empty() {
        return Err(AppError::validation(
            "market live tick event_id is required",
        ));
    }
    if tick.symbol_canonical.trim().is_empty() {
        return Err(AppError::validation(
            "market live tick symbol_canonical is required",
        ));
    }
    if valid_mark_price(tick.mark_price).is_none() {
        return Err(AppError::validation(
            "market live tick mark_price must be positive and finite",
        ));
    }
    Ok(())
}
