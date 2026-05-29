use super::direct_mode::run_direct_mode;
use super::*;

pub(super) async fn run_requested_mode(args: &Args) -> AppResult<Option<RunSummary>> {
    if let Some(summary) = run_direct_mode(args).await? {
        return Ok(Some(summary));
    }
    if has_retest_horizon_status_input(args) {
        return validate_retest_horizon_status_input(args).await.map(Some);
    }
    if let Some(path) = args.shadow_cycle_decision_file.as_deref() {
        return validate_shadow_cycle_decision_input(path).map(Some);
    }
    if args.build_shadow_cycle_decision {
        return build_shadow_cycle_decision_mode(args).await.map(Some);
    }
    Ok(None)
}

async fn validate_retest_horizon_status_input(args: &Args) -> AppResult<RunSummary> {
    let status = load_retest_horizon_status(args).await?;
    let validation = validate_retest_horizon_status(&status)?;
    Ok(RunSummary {
        retest_horizon_statuses_validated: 1,
        retest_cycle_scheduler_action: Some(validation.scheduler_action),
        retest_cycle_run_not_before_ms: validation.run_not_before_ms,
        ..RunSummary::default()
    })
}

fn validate_shadow_cycle_decision_input(path: &std::path::Path) -> AppResult<RunSummary> {
    let decision = read_shadow_cycle_decision(path)?;
    validate_shadow_cycle_decision(&decision)?;
    Ok(RunSummary {
        shadow_cycle_decisions_validated: 1,
        shadow_cycle_scheduler_action: Some(decision.scheduler_action),
        shadow_cycle_run_not_before_ms: decision.run_not_before_ms,
        shadow_cycle_focused_research_manifest_file: decision.focused_research_manifest_file,
        ..RunSummary::default()
    })
}
