use super::*;

mod dedupe;
mod location;
mod selection;
#[cfg(test)]
mod tests;

pub(in crate::cli) use dedupe::{
    append_unique_bundles, append_unique_oss_adapter_runs, append_unique_replay_runs,
    append_unique_shadow_validation_runs, filter_historical_replay_runs_for_current_research,
};
pub(in crate::cli) use location::parse_s3_uri;
use location::{ReplayRunLocation, replay_run_location};
pub(in crate::cli) use selection::append_indexed_replay_runs;

pub(in crate::cli) async fn load_replay_runs_from_index_records(
    records: &[ReplayRunIndexRecord],
) -> AppResult<Vec<ReplayRun>> {
    let mut local_locations = BTreeMap::<PathBuf, BTreeSet<String>>::new();
    let mut s3_locations = BTreeMap::<(String, String), BTreeSet<String>>::new();

    for record in records {
        match replay_run_location(record)? {
            ReplayRunLocation::Local(path) => {
                local_locations
                    .entry(path)
                    .or_default()
                    .insert(record.replay_run_id.clone());
            }
            ReplayRunLocation::S3 { bucket, key } => {
                s3_locations
                    .entry((bucket, key))
                    .or_default()
                    .insert(record.replay_run_id.clone());
            }
        }
    }

    let mut replay_runs = Vec::new();
    for (path, expected_ids) in local_locations {
        let runs = read_replay_runs(&path)?;
        append_indexed_replay_runs(
            &mut replay_runs,
            runs,
            &expected_ids,
            &path.display().to_string(),
        )?;
    }
    for ((bucket, key), expected_ids) in s3_locations {
        let runs = read_replay_runs_from_s3(&bucket, std::slice::from_ref(&key)).await?;
        append_indexed_replay_runs(
            &mut replay_runs,
            runs,
            &expected_ids,
            &format!("s3://{bucket}/{key}"),
        )?;
    }
    Ok(replay_runs)
}
