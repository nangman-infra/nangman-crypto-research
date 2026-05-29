mod context;
mod decision;
mod horizon;
mod output;
mod schedule;
mod stage;

use crate::error::AppResult;
use serde_json::Value;

use super::RetestHorizonStatusBuildOptions;
use super::status_parts::validate_plan;
use context::prepare_context;
use decision::build_decision_state;
use horizon::build_horizon_summary;
use output::build_status_json;
use schedule::build_materialization_schedule;
use stage::build_stage_state;

pub fn build_retest_horizon_status(
    plan: &Value,
    driver_summary: Option<&Value>,
    options: &RetestHorizonStatusBuildOptions,
) -> AppResult<Value> {
    validate_plan(plan)?;
    let context = prepare_context(plan, driver_summary);
    let stage = build_stage_state(&context);
    let horizon = build_horizon_summary(&context);
    let schedule = build_materialization_schedule(&context);
    let decision = build_decision_state(&stage, &horizon);

    Ok(build_status_json(
        options, &context, &stage, &horizon, &schedule, &decision,
    ))
}
