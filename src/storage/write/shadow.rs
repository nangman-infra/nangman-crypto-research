use crate::error::{AppError, AppResult};
use crate::model::ShadowCycleDecision;
use crate::storage::client::s3_client;
use crate::storage::objects::put_object_json;
use crate::storage::partition::{normalize_prefix, partition};

pub async fn write_shadow_cycle_decision_to_s3(
    bucket: &str,
    prefix: &str,
    decision: &ShadowCycleDecision,
    output_partition_at_ms: i64,
) -> AppResult<String> {
    if bucket.trim().is_empty() {
        return Err(AppError::config(
            "shadow cycle decision output S3 bucket must not be empty",
        ));
    }
    let client = s3_client().await?;
    let dt = partition(output_partition_at_ms)?;
    let prefix = normalize_prefix(prefix);
    let key = format!(
        "{prefix}shadow-cycle-decision/schema={}/dt={}/hour={:02}/decision_id={}/decision.json",
        decision.schema_version, dt.date, dt.hour, decision.decision_id
    );
    put_object_json(&client, bucket, &key, decision).await?;
    Ok(format!("s3://{bucket}/{key}"))
}
