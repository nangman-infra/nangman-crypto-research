use aws_sdk_s3::Client;

use crate::artifacts::build_replay_run_index_records;
use crate::error::AppResult;
use crate::io::ResearchOutputArtifacts;

use super::jsonl::write_jsonl_dataset;
use super::keys::{ResearchOutputS3Keys, s3_uri};

pub(super) async fn write_replay_outputs(
    client: &Client,
    bucket: &str,
    keys: &ResearchOutputS3Keys,
    artifacts: &ResearchOutputArtifacts<'_>,
    written: &mut Vec<String>,
) -> AppResult<()> {
    if artifacts.replay_runs.is_empty() {
        return Ok(());
    }

    let replay_key = keys.jsonl_dataset("replay-run", &artifacts.replay_runs[0].schema_version);
    let replay_run_uri = s3_uri(bucket, &replay_key);
    let replay_run_index_records = build_replay_run_index_records(
        artifacts.report,
        artifacts.replay_runs,
        &replay_run_uri,
        Some(bucket),
        Some(&replay_key),
    );

    write_jsonl_dataset(
        client,
        bucket,
        keys,
        "replay-run",
        &artifacts.replay_runs[0].schema_version,
        artifacts.replay_runs,
        written,
    )
    .await?;
    write_jsonl_dataset(
        client,
        bucket,
        keys,
        "replay-run-index",
        &replay_run_index_records[0].schema_version,
        &replay_run_index_records,
        written,
    )
    .await
}
