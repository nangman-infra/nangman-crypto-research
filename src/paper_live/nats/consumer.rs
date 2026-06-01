use super::*;
use crate::error::AppError;
use async_nats::jetstream;
use async_nats::jetstream::consumer::AckPolicy;
use async_nats::jetstream::consumer::PullConsumer;
use async_nats::jetstream::stream::Stream;
use std::time::Duration;

pub(super) async fn market_live_consumer(
    stream: &Stream,
    config: &MarketLiveNatsConfig,
) -> AppResult<PullConsumer> {
    stream
        .get_or_create_consumer(
            &config.consumer,
            jetstream::consumer::pull::Config {
                durable_name: Some(config.consumer.clone()),
                filter_subject: config.subject.clone(),
                ack_policy: AckPolicy::Explicit,
                ack_wait: Duration::from_secs(config.ack_wait_secs),
                max_ack_pending: config.batch_size as i64,
                deliver_policy: super::super::config::deliver_policy(&config.deliver_policy)?,
                ..Default::default()
            },
        )
        .await
        .map_err(|error| {
            AppError::nats(format!(
                "get/create consumer {} on stream {}: {error}",
                config.consumer, config.stream
            ))
        })
}

pub(super) async fn delete_market_live_consumer(
    stream: &Stream,
    config: &MarketLiveNatsConfig,
) -> AppResult<()> {
    if !config.delete_consumer_after_read {
        return Ok(());
    }
    stream
        .delete_consumer(&config.consumer)
        .await
        .map(|_| ())
        .map_err(|error| {
            AppError::nats(format!(
                "delete market live consumer {} on stream {}: {error}",
                config.consumer, config.stream
            ))
        })
}
