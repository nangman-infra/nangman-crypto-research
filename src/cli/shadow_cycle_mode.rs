use super::*;

mod accumulation;
mod focused_dispatch;
mod latest_state;
mod output;

#[cfg(test)]
pub(super) use accumulation::build_shadow_accumulation_manifest_dispatch;
use focused_dispatch::try_apply_focused_shadow_dispatch;
use latest_state::load_latest_shadow_runs;
use output::append_output_files;

pub(super) use output::write_shadow_cycle_decision_outputs;

pub(super) async fn run_shadow_cycle_from_latest_state_mode(args: &Args) -> AppResult<RunSummary> {
    let output_partition_at_ms = args.now_ms.unwrap_or_else(now_ms);
    let shadow_runs = load_latest_shadow_runs(args).await?;
    let latest_l1_as_of_ms = shadow_cycle_latest_l1_as_of_ms(args).await?;
    let mut decision =
        build_shadow_cycle_decision(&shadow_runs, latest_l1_as_of_ms, output_partition_at_ms);
    let dispatch_summary = try_apply_focused_shadow_dispatch(
        args,
        &shadow_runs,
        latest_l1_as_of_ms,
        output_partition_at_ms,
        &mut decision,
    )
    .await?;
    validate_shadow_cycle_decision(&decision)?;
    let output_files = append_output_files(
        dispatch_summary.output_files,
        write_shadow_cycle_decision_outputs(args, &decision, output_partition_at_ms).await?,
    );
    emit_shadow_cycle_decision_alert_from_env(&decision).await;

    Ok(RunSummary {
        shadow_cycle_decisions_validated: 1,
        shadow_cycle_decisions_created: 1,
        shadow_cycle_scheduler_action: Some(decision.scheduler_action),
        shadow_cycle_run_not_before_ms: decision.run_not_before_ms,
        shadow_cycle_focused_research_manifest_file: decision.focused_research_manifest_file,
        focused_retest_manifests_created: dispatch_summary.focused_retest_manifests_created,
        focused_retest_horizon_count: dispatch_summary.focused_retest_horizon_count,
        focused_retest_candidate_bundle_refs: dispatch_summary.focused_retest_candidate_bundle_refs,
        shadow_validation_runs_loaded: shadow_runs.len(),
        output_files,
        ..RunSummary::default()
    })
}
