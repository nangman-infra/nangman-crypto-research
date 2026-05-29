use serde_json::{Value, json};

use super::super::status_parts::{iso8601_ms, max_ms_for_action, min_ms_for_action};
use super::context::StatusBuildContext;

pub(super) struct MaterializationSchedule {
    pub(super) value: Value,
    pub(super) next_wait_due_ms: Option<i64>,
    pub(super) wait_deficit_ms: Option<i64>,
}

pub(super) fn build_materialization_schedule(
    context: &StatusBuildContext,
) -> MaterializationSchedule {
    let next_wait_due_ms = min_ms_for_action(
        &context.rows,
        "wait_for_market_l1_horizon",
        "horizon_due_ms",
    );
    let last_wait_due_ms = max_ms_for_action(
        &context.rows,
        "wait_for_market_l1_horizon",
        "horizon_due_ms",
    );
    let oldest_accumulation_due_ms = min_ms_for_action(
        &context.rows,
        "accumulate_completed_native_replay_samples",
        "horizon_due_ms",
    );
    let latest_accumulation_due_ms = max_ms_for_action(
        &context.rows,
        "accumulate_completed_native_replay_samples",
        "horizon_due_ms",
    );
    let wait_deficit_ms = match (context.latest_l1_as_of_ms, next_wait_due_ms) {
        (Some(latest), Some(next)) => Some((next - latest).max(0)),
        _ => None,
    };
    let value = json!({
        "latest_l1_as_of_ms": context.latest_l1_as_of_ms,
        "latest_l1_as_of_iso": context.latest_l1_as_of_ms.map(iso8601_ms),
        "next_wait_horizon_due_ms": next_wait_due_ms,
        "next_wait_horizon_due_iso": next_wait_due_ms.map(iso8601_ms),
        "last_wait_horizon_due_ms": last_wait_due_ms,
        "last_wait_horizon_due_iso": last_wait_due_ms.map(iso8601_ms),
        "next_wait_deficit_ms": wait_deficit_ms,
        "oldest_accumulation_due_ms": oldest_accumulation_due_ms,
        "oldest_accumulation_due_iso": oldest_accumulation_due_ms.map(iso8601_ms),
        "latest_accumulation_due_ms": latest_accumulation_due_ms,
        "latest_accumulation_due_iso": latest_accumulation_due_ms.map(iso8601_ms)
    });

    MaterializationSchedule {
        value,
        next_wait_due_ms,
        wait_deficit_ms,
    }
}
