use super::config::AlertConfig;
use super::event::AlertEvent;
use crate::time::now_ms;
use aws_sdk_s3::primitives::ByteStream;

mod build;
mod client;
mod key;
mod payload;

pub(super) use build::build_pipeline_alert_delivery;
use client::s3_client;
#[cfg(test)]
pub(super) use key::{pipeline_alert_event_key, s3_key_token};
#[cfg(test)]
pub(super) use payload::PipelineAlertEvent;

pub(super) async fn send_event(config: &AlertConfig, event: &AlertEvent) -> Result<(), String> {
    let delivery = build_pipeline_alert_delivery(config, event, now_ms())?;
    s3_client()
        .await?
        .put_object()
        .bucket(&config.event_bucket)
        .key(delivery.key)
        .content_type("application/json")
        .body(ByteStream::from(delivery.body))
        .send()
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}
