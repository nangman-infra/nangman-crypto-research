use crate::model::{
    ABSOLUTE_MAX_HOLDING_HOURS, HOLDING_POLICY_VERSION, HoldingPolicy, TARGET_MAX_HOLDING_HOURS,
};

const HOUR_MS: i64 = 60 * 60 * 1000;

pub fn default_holding_policy(decision_available_at_ms: i64) -> HoldingPolicy {
    HoldingPolicy {
        target_max_holding_hours: TARGET_MAX_HOLDING_HOURS,
        absolute_max_holding_hours: ABSOLUTE_MAX_HOLDING_HOURS,
        absolute_exit_deadline_ms: decision_available_at_ms
            + i64::from(ABSOLUTE_MAX_HOLDING_HOURS) * HOUR_MS,
        force_flat_policy: "daily_or_ttl_exit".to_owned(),
        overnight_risk_exception: false,
        holding_policy_version: HOLDING_POLICY_VERSION.to_owned(),
    }
}

pub fn horizon_within_absolute_limit(duration_ms: i64) -> bool {
    duration_ms <= i64::from(ABSOLUTE_MAX_HOLDING_HOURS) * HOUR_MS
}
