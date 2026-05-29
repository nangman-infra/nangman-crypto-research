use super::{
    AppResult, Args, ReplayRun, ResearchInputs, enforce_budget,
    filter_historical_replay_runs_for_current_research, load_historical_replay_runs,
};

pub(super) async fn load_current_historical_replay_runs(
    args: &Args,
    inputs: &ResearchInputs,
    replay_runs: &[ReplayRun],
) -> AppResult<Vec<ReplayRun>> {
    let historical_replay_runs = filter_historical_replay_runs_for_current_research(
        load_historical_replay_runs(
            args,
            inputs.manifest.as_ref(),
            inputs.budget.max_historical_replay_run_ref_count,
        )
        .await?,
        replay_runs,
    );
    enforce_budget(
        "historical_replay_run_count",
        historical_replay_runs.len(),
        inputs.budget.max_replay_run_count,
    )?;
    Ok(historical_replay_runs)
}

pub(super) fn aggregate_replay_runs(
    inputs: &ResearchInputs,
    replay_runs: &[ReplayRun],
    historical_replay_runs: &[ReplayRun],
) -> AppResult<Vec<ReplayRun>> {
    enforce_budget(
        "oss_adapter_run_count",
        inputs.oss_adapter_runs.len(),
        inputs.budget.max_oss_adapter_run_ref_count,
    )?;
    enforce_budget(
        "shadow_validation_run_count",
        inputs.shadow_validation_runs.len(),
        inputs.budget.max_shadow_validation_run_ref_count,
    )?;
    let mut aggregate_replay_runs = historical_replay_runs.to_vec();
    aggregate_replay_runs.extend(replay_runs.iter().cloned());
    enforce_budget(
        "aggregate_replay_run_count",
        aggregate_replay_runs.len(),
        inputs.budget.max_replay_run_count,
    )?;
    Ok(aggregate_replay_runs)
}
