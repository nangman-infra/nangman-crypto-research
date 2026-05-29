use aws_sdk_s3::Client;

use crate::error::AppResult;
use crate::model::ResearchRunReport;
use crate::storage::objects::put_object_json;

use super::jsonl::write_jsonl_dataset;
use super::keys::{ResearchOutputS3Keys, s3_uri};

pub(super) async fn write_portfolio_outputs(
    client: &Client,
    bucket: &str,
    keys: &ResearchOutputS3Keys,
    report: &ResearchRunReport,
    written: &mut Vec<String>,
) -> AppResult<()> {
    if let Some(snapshot) = report.portfolio_allocation_snapshot.as_ref() {
        let snapshot_key = keys.json_object(
            "portfolio-allocation-snapshot",
            &snapshot.schema_version,
            "snapshot.json",
        );
        put_object_json(client, bucket, &snapshot_key, snapshot).await?;
        written.push(s3_uri(bucket, &snapshot_key));
    }

    if !report.portfolio_risk_reject_events.is_empty() {
        write_jsonl_dataset(
            client,
            bucket,
            keys,
            "portfolio-risk-reject-event",
            &report.portfolio_risk_reject_events[0].schema_version,
            &report.portfolio_risk_reject_events,
            written,
        )
        .await?;
    }

    if !report.portfolio_reduce_only_signals.is_empty() {
        write_jsonl_dataset(
            client,
            bucket,
            keys,
            "portfolio-reduce-only-signal",
            &report.portfolio_reduce_only_signals[0].schema_version,
            &report.portfolio_reduce_only_signals,
            written,
        )
        .await?;
    }

    Ok(())
}
