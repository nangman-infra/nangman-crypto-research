use chrono::{DateTime, SecondsFormat, Utc};

use crate::model::ShadowValidationRun;

use super::super::MS_PER_HOUR;

pub(super) fn target_exit_deadline_ms(run: &ShadowValidationRun) -> Option<i64> {
    let absolute_deadline = run.holding_policy.absolute_exit_deadline_ms;
    let absolute_hours = i64::from(run.holding_policy.absolute_max_holding_hours);
    let target_hours = i64::from(run.holding_policy.target_max_holding_hours);
    if absolute_deadline <= 0 || absolute_hours <= 0 || target_hours <= 0 {
        return None;
    }
    Some(absolute_deadline - (absolute_hours * MS_PER_HOUR) + (target_hours * MS_PER_HOUR))
}

pub(super) fn min_optional_ms(left: Option<i64>, right: Option<i64>) -> Option<i64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    }
}

pub(super) fn iso8601_ms(value: i64) -> String {
    DateTime::<Utc>::from_timestamp_millis(value)
        .unwrap_or(DateTime::<Utc>::UNIX_EPOCH)
        .to_rfc3339_opts(SecondsFormat::Secs, true)
}
