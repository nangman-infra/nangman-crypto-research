use crate::error::{AppError, AppResult};
use async_nats::jetstream::consumer::DeliverPolicy;

pub const DEFAULT_MARKET_LIVE_NATS_STREAM: &str = "MARKET_LIVE";
pub const DEFAULT_MARKET_LIVE_NATS_SUBJECT: &str = "market_live_tick.created.>";
pub const DEFAULT_MARKET_LIVE_NATS_CONSUMER: &str = "research-paper-watch-live";
pub const DEFAULT_MARKET_LIVE_NATS_DELIVER_POLICY: &str = "last_per_subject";
pub const DEFAULT_MARKET_LIVE_NATS_BATCH_SIZE: usize = 100;
pub const DEFAULT_MARKET_LIVE_NATS_MAX_MESSAGES: usize = 500;
pub const DEFAULT_MARKET_LIVE_NATS_ACK_WAIT_SECS: u64 = 30;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketLiveNatsConfig {
    pub url: String,
    pub stream: String,
    pub subject: String,
    pub consumer: String,
    pub deliver_policy: String,
    pub batch_size: usize,
    pub max_messages: usize,
    pub ack_wait_secs: u64,
    pub delete_consumer_after_read: bool,
}

pub(super) fn validate_nats_config(config: &MarketLiveNatsConfig) -> AppResult<()> {
    if !config.url.starts_with("nats://") {
        return Err(AppError::config(
            "market live NATS url must start with nats://",
        ));
    }
    if config.stream.trim().is_empty() {
        return Err(AppError::config(
            "market live NATS stream must not be empty",
        ));
    }
    if config.subject.trim().is_empty() {
        return Err(AppError::config(
            "market live NATS subject must not be empty",
        ));
    }
    if config.consumer.trim().is_empty() {
        return Err(AppError::config(
            "market live NATS consumer must not be empty",
        ));
    }
    if config.batch_size == 0 || config.max_messages == 0 || config.ack_wait_secs == 0 {
        return Err(AppError::config(
            "market live NATS batch size, max messages, and ack wait must be positive",
        ));
    }
    deliver_policy(&config.deliver_policy)?;
    Ok(())
}

pub(super) fn deliver_policy(value: &str) -> AppResult<DeliverPolicy> {
    match value {
        "all" => Ok(DeliverPolicy::All),
        "new" => Ok(DeliverPolicy::New),
        "last" => Ok(DeliverPolicy::Last),
        "last_per_subject" => Ok(DeliverPolicy::LastPerSubject),
        other => Err(AppError::config(format!(
            "unsupported market live deliver policy: {other}"
        ))),
    }
}
