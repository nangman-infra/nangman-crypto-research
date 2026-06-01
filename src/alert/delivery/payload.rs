use crate::alert::config::APP_NAME;
use crate::alert::event::AlertEvent;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub(in crate::alert) struct PipelineAlertEvent<'a> {
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
