use super::validation::{output_prefix, validate_key_component};
use crate::error::{AppError, AppResult};
use crate::model::{ResearchInputManifest, RetestCycleSourceState};
use crate::storage::partition::partition;

const RESEARCH_INPUT_MANIFEST_PREFIX: &str =
    "research-input-manifest/schema=research_input_manifest_v1";
const RETEST_CYCLE_SOURCE_STATE_PREFIX: &str =
    "retest-cycle-source-state/schema=research_retest_cycle_source_state_v1";
const RETEST_HORIZON_PLAN_PREFIX: &str =
    "retest-horizon-plan/schema=research_retest_horizon_plan_v1";
const RETEST_HORIZON_STATUS_PREFIX: &str =
    "retest-horizon-status/schema=research_horizon_status_checkpoint_v1";

pub(in crate::storage::write::retest) fn research_input_manifest_key(
    prefix: &str,
    manifest: &ResearchInputManifest,
    output_partition_at_ms: i64,
) -> AppResult<String> {
    let packet_id = manifest
        .research_packet_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            AppError::validation(
                "research input manifest requires research_packet_id for S3 output",
            )
        })?;
    validate_key_component(packet_id, "research input manifest research_packet_id")?;
    let dt = partition(output_partition_at_ms)?;
    let prefix = output_prefix(
        prefix,
        RESEARCH_INPUT_MANIFEST_PREFIX,
        "research-input-manifest/",
        "focused retest manifest S3 prefix must start with research-input-manifest/",
    )?;
    Ok(format!(
        "{prefix}dt={}/hour={:02}/research_packet_id={packet_id}/manifest.json",
        dt.date, dt.hour
    ))
}

pub(in crate::storage::write::retest) fn retest_cycle_source_state_key(
    prefix: &str,
    state: &RetestCycleSourceState,
    output_partition_at_ms: i64,
) -> AppResult<String> {
    validate_key_component(
        &state.research_packet_id,
        "retest cycle source state research_packet_id",
    )?;
    validate_key_component(
        &state.source_research_report_id,
        "retest cycle source state source_research_report_id",
    )?;
    let dt = partition(output_partition_at_ms)?;
    let prefix = output_prefix(
        prefix,
        RETEST_CYCLE_SOURCE_STATE_PREFIX,
        "retest-cycle-source-state/",
        "retest cycle source state S3 prefix must start with retest-cycle-source-state/",
    )?;
    Ok(format!(
        "{prefix}dt={}/hour={:02}/research_packet_id={}/research_run_report_id={}/state.json",
        dt.date, dt.hour, state.research_packet_id, state.source_research_report_id
    ))
}

pub(in crate::storage::write::retest) fn retest_horizon_plan_key(
    prefix: &str,
    plan: &serde_json::Value,
    output_partition_at_ms: i64,
) -> AppResult<String> {
    let dt = partition(output_partition_at_ms)?;
    let prefix = output_prefix(
        prefix,
        RETEST_HORIZON_PLAN_PREFIX,
        "retest-horizon-plan/",
        "retest horizon plan S3 prefix must start with retest-horizon-plan/",
    )?;
    let generated_at_ms = generated_at_ms(plan, output_partition_at_ms);
    Ok(format!(
        "{prefix}dt={}/hour={:02}/generated_at_ms={generated_at_ms}/retest-horizon-plan.json",
        dt.date, dt.hour
    ))
}

pub(in crate::storage::write::retest) fn retest_horizon_status_key(
    prefix: &str,
    status: &serde_json::Value,
    output_partition_at_ms: i64,
) -> AppResult<String> {
    let dt = partition(output_partition_at_ms)?;
    let prefix = output_prefix(
        prefix,
        RETEST_HORIZON_STATUS_PREFIX,
        "retest-horizon-status/",
        "retest horizon status S3 prefix must start with retest-horizon-status/",
    )?;
    let generated_at_ms = generated_at_ms(status, output_partition_at_ms);
    Ok(format!(
        "{prefix}dt={}/hour={:02}/generated_at_ms={generated_at_ms}/retest-horizon-status.json",
        dt.date, dt.hour
    ))
}

fn generated_at_ms(value: &serde_json::Value, fallback_ms: i64) -> i64 {
    value
        .get("generated_at_ms")
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(fallback_ms)
}
