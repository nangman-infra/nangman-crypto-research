use aws_sdk_s3::Client;

use crate::error::AppResult;
use crate::io::ResearchOutputArtifacts;

use super::jsonl::write_jsonl_dataset;
use super::keys::ResearchOutputS3Keys;

pub(super) async fn write_paper_outputs(
    client: &Client,
    bucket: &str,
    keys: &ResearchOutputS3Keys,
    artifacts: &ResearchOutputArtifacts<'_>,
    written: &mut Vec<String>,
) -> AppResult<()> {
    if !artifacts.paper_watch_candidates.is_empty() {
        write_jsonl_dataset(
            client,
            bucket,
            keys,
            "paper-watch-candidate",
            &artifacts.paper_watch_candidates[0].schema_version,
            artifacts.paper_watch_candidates,
            written,
        )
        .await?;
    }

    if !artifacts.paper_trade_candidates.is_empty() {
        write_jsonl_dataset(
            client,
            bucket,
            keys,
            "paper-trade-candidate",
            &artifacts.paper_trade_candidates[0].schema_version,
            artifacts.paper_trade_candidates,
            written,
        )
        .await?;
    }

    if !artifacts.paper_trade_runs.is_empty() {
        write_jsonl_dataset(
            client,
            bucket,
            keys,
            "paper-trade-run",
            &artifacts.paper_trade_runs[0].schema_version,
            artifacts.paper_trade_runs,
            written,
        )
        .await?;
    }

    if !artifacts.paper_trade_summaries.is_empty() {
        write_jsonl_dataset(
            client,
            bucket,
            keys,
            "paper-trade-summary",
            &artifacts.paper_trade_summaries[0].schema_version,
            artifacts.paper_trade_summaries,
            written,
        )
        .await?;
    }

    if !artifacts.paper_trade_marks.is_empty() {
        write_jsonl_dataset(
            client,
            bucket,
            keys,
            "paper-trade-mark",
            &artifacts.paper_trade_marks[0].schema_version,
            artifacts.paper_trade_marks,
            written,
        )
        .await?;
    }

    Ok(())
}
