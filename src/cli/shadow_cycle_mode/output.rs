use super::*;

pub(in crate::cli) async fn write_shadow_cycle_decision_outputs(
    args: &Args,
    decision: &crate::model::ShadowCycleDecision,
    output_partition_at_ms: i64,
) -> AppResult<Vec<String>> {
    if let Some(output_file) = args.shadow_cycle_decision_output_file.as_deref() {
        return write_shadow_cycle_decision(output_file, decision)
            .map(|path| vec![path.display().to_string()]);
    }
    if let Some(output_dir) = args.output_dir.as_deref() {
        return write_shadow_cycle_decision_to_dir(output_dir, decision, output_partition_at_ms)
            .map(|path| vec![path.display().to_string()]);
    }
    if let Some(output_bucket) = args.output_s3_bucket.as_deref() {
        return write_shadow_cycle_decision_to_s3(
            output_bucket,
            args.output_s3_prefix.as_deref().unwrap_or(""),
            decision,
            output_partition_at_ms,
        )
        .await
        .map(|uri| vec![uri]);
    }
    Err(AppError::config(
        "shadow cycle decision output target is required",
    ))
}

pub(super) fn append_output_files(mut left: Vec<String>, right: Vec<String>) -> Vec<String> {
    left.extend(right);
    left
}
