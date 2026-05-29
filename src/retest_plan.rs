mod action;
mod output;
mod row;
mod types;

use crate::error::{AppError, AppResult};
use crate::model::{
    IntelCandidateEvidenceBundle, RETEST_HORIZON_PLAN_SCHEMA_VERSION, ResearchRunReport,
};
use serde_json::{Value, json};

pub use types::RetestHorizonPlanBuildOptions;

pub fn build_retest_horizon_plan(
    bundles: &[IntelCandidateEvidenceBundle],
    report: &ResearchRunReport,
    options: &RetestHorizonPlanBuildOptions,
) -> AppResult<Value> {
    if bundles.is_empty() {
        return Err(AppError::validation(
            "retest horizon plan requires at least one candidate bundle",
        ));
    }
    if report.research_gate_policy.min_completed_samples_for_shadow == 0 {
        return Err(AppError::validation(
            "research gate policy min_completed_samples_for_shadow must be greater than zero",
        ));
    }

    let rows = bundles
        .iter()
        .flat_map(|bundle| {
            bundle
                .allowed_horizons
                .iter()
                .map(|horizon| row::build_row(bundle, horizon, report, options.latest_l1_as_of_ms))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    if rows.is_empty() {
        return Err(AppError::validation(
            "retest horizon plan requires at least one candidate horizon",
        ));
    }

    Ok(json!({
        "schema_version": RETEST_HORIZON_PLAN_SCHEMA_VERSION,
        "generated_at_ms": options.generated_at_ms,
        "manifest_file": options.manifest_label,
        "report_file": options.report_label,
        "latest_l1_as_of_ms": options.latest_l1_as_of_ms,
        "research_gate_policy": report.research_gate_policy,
        "summary": output::summary(&rows, bundles.len()),
        "by_candidate": output::by_candidate(&rows),
        "horizon_rows": rows.iter().map(output::row_json).collect::<Vec<_>>()
    }))
}
