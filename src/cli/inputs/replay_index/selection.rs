use super::*;

pub(in crate::cli) fn append_indexed_replay_runs(
    target: &mut Vec<ReplayRun>,
    runs: Vec<ReplayRun>,
    expected_ids: &BTreeSet<String>,
    label: &str,
) -> AppResult<()> {
    let mut matched_ids = BTreeSet::new();
    let selected_runs = runs
        .into_iter()
        .filter(|run| {
            let matched = expected_ids.contains(&run.replay_run_id);
            if matched {
                matched_ids.insert(run.replay_run_id.clone());
            }
            matched
        })
        .collect::<Vec<_>>();

    let missing_ids = expected_ids
        .difference(&matched_ids)
        .cloned()
        .collect::<Vec<_>>();
    if !missing_ids.is_empty() {
        return Err(AppError::validation(format!(
            "replay_run_index points to missing replay_run_id(s) in {label}: {}",
            missing_ids.join(",")
        )));
    }

    append_unique_replay_runs(target, selected_runs);
    Ok(())
}
