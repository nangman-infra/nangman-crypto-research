use std::env;

pub(super) const APP_NAME: &str = "research-app";
pub(super) const DEFAULT_ENVIRONMENT: &str = "dev";
pub(super) const DEFAULT_PIPELINE_ALERT_S3_PREFIX: &str =
    "pipeline-alert-event/schema=pipeline_alert_event_v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertPriority {
    P0,
    P1,
    P2,
    P3,
}

impl AlertPriority {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::P0 => "P0",
            Self::P1 => "P1",
            Self::P2 => "P2",
            Self::P3 => "P3",
        }
    }

    pub(super) fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_uppercase().as_str() {
            "P0" => Some(Self::P0),
            "P1" => Some(Self::P1),
            "P2" => Some(Self::P2),
            "P3" => Some(Self::P3),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct AlertConfig {
    pub(in crate::alert) event_bucket: String,
    pub(in crate::alert) event_prefix: String,
    pub(in crate::alert) environment: String,
    pub(in crate::alert) min_priority: AlertPriority,
    pub(in crate::alert) include_retest_summary: bool,
    pub(in crate::alert) include_shadow_wait: bool,
}

impl AlertConfig {
    pub(super) fn from_env() -> Option<Self> {
        let event_bucket = env::var("NANGMAN_PIPELINE_ALERT_S3_BUCKET")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .or_else(|| {
                env::var("RESEARCH_OUTPUT_S3_BUCKET")
                    .ok()
                    .filter(|value| !value.trim().is_empty())
            })?;
        let event_prefix = env::var("NANGMAN_PIPELINE_ALERT_S3_PREFIX")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_PIPELINE_ALERT_S3_PREFIX.to_owned());
        let environment =
            env::var("NANGMAN_ALERT_ENV").unwrap_or_else(|_| DEFAULT_ENVIRONMENT.to_owned());
        let min_priority = env::var("NANGMAN_ALERT_MIN_PRIORITY")
            .ok()
            .and_then(|value| AlertPriority::parse(&value))
            .unwrap_or(AlertPriority::P2);
        let include_retest_summary = env_bool("NANGMAN_ALERT_INCLUDE_RETEST_SUMMARY");
        let include_shadow_wait = env_bool("NANGMAN_ALERT_INCLUDE_SHADOW_WAIT");

        Some(Self {
            event_bucket,
            event_prefix,
            environment,
            min_priority,
            include_retest_summary,
            include_shadow_wait,
        })
    }

    pub(in crate::alert) fn allows(&self, priority: AlertPriority) -> bool {
        priority <= self.min_priority
    }
}

fn env_bool(name: &str) -> bool {
    env::var(name)
        .ok()
        .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
}
