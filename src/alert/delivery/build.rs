use super::key::pipeline_alert_event_key;
use super::payload::PipelineAlertEvent;
use crate::alert::config::{APP_NAME, AlertConfig};
use crate::alert::event::AlertEvent;
use crate::hash::stable_id;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::alert) struct PipelineAlertDelivery {
    pub(in crate::alert) key: String,
    pub(in crate::alert) body: Vec<u8>,
}

pub(in crate::alert) fn build_pipeline_alert_delivery(
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
