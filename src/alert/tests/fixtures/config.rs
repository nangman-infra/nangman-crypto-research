use super::super::*;

pub(crate) fn test_config(min_priority: AlertPriority) -> AlertConfig {
    AlertConfig {
        event_bucket: "nangman-crypto-dev-research-962214".to_owned(),
        event_prefix: DEFAULT_PIPELINE_ALERT_S3_PREFIX.to_owned(),
        environment: "dev".to_owned(),
        min_priority,
        include_retest_summary: false,
        include_shadow_wait: false,
    }
}
