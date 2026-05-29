use super::config::{APP_NAME, AlertConfig};
use super::event::AlertEvent;
use crate::{hash::stable_id, time::now_ms};
use aws_config::BehaviorVersion;
use aws_sdk_s3::{Client, primitives::ByteStream};
use aws_types::region::Region;
use chrono::{DateTime, Datelike, Timelike, Utc};
use serde::Serialize;
use std::env;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PipelineAlertDelivery {
    pub(in crate::alert) key: String,
    pub(in crate::alert) body: Vec<u8>,
}

pub(super) fn build_pipeline_alert_delivery(
    config: &AlertConfig,
    event: &AlertEvent,
    created_at_ms: i64,
) -> Result<PipelineAlertDelivery, String> {
    let priority = event.priority.as_str();
    let event_id = stable_id(
        "pipeline_alert",
        &[APP_NAME, priority, &event.title, &created_at_ms.to_string()],
    );
    let dedupe_key = stable_id(
        "pipeline_alert_dedupe",
        &[
            APP_NAME,
            priority,
            &event.title,
            &event.conclusion,
            &event.current_state.join("\n"),
            &event.reasons.join("\n"),
        ],
    );
    let payload = PipelineAlertEvent::from_alert_event(
        event,
        &event_id,
        &dedupe_key,
        &config.environment,
        created_at_ms,
    );
    let key = pipeline_alert_event_key(
        &config.event_prefix,
        created_at_ms,
        APP_NAME,
        priority,
        &event_id,
    )?;
    let body = serde_json::to_vec_pretty(&payload).map_err(|error| error.to_string())?;
    Ok(PipelineAlertDelivery { key, body })
}

#[derive(Debug, Serialize)]
pub(super) struct PipelineAlertEvent<'a> {
    schema_version: &'static str,
    event_id: &'a str,
    dedupe_key: &'a str,
    app: &'static str,
    environment: &'a str,
    priority: &'static str,
    title: &'a str,
    conclusion: &'a str,
    current_state: &'a [String],
    reasons: &'a [String],
    next_actions: &'a [String],
    safety: &'a [String],
    created_at_ms: i64,
}

impl<'a> PipelineAlertEvent<'a> {
    pub(in crate::alert) fn from_alert_event(
        event: &'a AlertEvent,
        event_id: &'a str,
        dedupe_key: &'a str,
        environment: &'a str,
        created_at_ms: i64,
    ) -> Self {
        Self {
            schema_version: "pipeline_alert_event_v1",
            event_id,
            dedupe_key,
            app: APP_NAME,
            environment,
            priority: event.priority.as_str(),
            title: &event.title,
            conclusion: &event.conclusion,
            current_state: &event.current_state,
            reasons: &event.reasons,
            next_actions: &event.next_actions,
            safety: &event.safety,
            created_at_ms,
        }
    }
}

async fn s3_client() -> Result<Client, String> {
    let mut loader = aws_config::defaults(BehaviorVersion::latest());
    if let Some(region) = env_string("AWS_REGION").or_else(|| env_string("AWS_DEFAULT_REGION")) {
        loader = loader.region(Region::new(region));
    }
    let config = loader.load().await;
    Ok(Client::new(&config))
}

pub(super) fn pipeline_alert_event_key(
    prefix: &str,
    created_at_ms: i64,
    app: &str,
    priority: &str,
    event_id: &str,
) -> Result<String, String> {
    let created_at = DateTime::<Utc>::from_timestamp_millis(created_at_ms)
        .ok_or_else(|| "created_at_ms is outside supported timestamp range".to_owned())?;
    Ok(format!(
        "{}/dt={:04}-{:02}-{:02}/hour={:02}/app={}/priority={}/{}.json",
        prefix.trim().trim_matches('/'),
        created_at.year(),
        created_at.month(),
        created_at.day(),
        created_at.hour(),
        s3_key_token(app),
        s3_key_token(priority),
        s3_key_token(event_id),
    ))
}

pub(super) fn s3_key_token(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '=' => character,
            _ => '_',
        })
        .collect()
}

fn env_string(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}
