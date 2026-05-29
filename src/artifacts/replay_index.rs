use crate::hash::stable_id;
use crate::model::{
    REPLAY_RUN_INDEX_SCHEMA_VERSION, ReplayRun, ReplayRunIndexRecord, ResearchRunReport,
};

pub fn build_replay_run_index_records(
    report: &ResearchRunReport,
    replay_runs: &[ReplayRun],
    replay_run_uri: &str,
    replay_run_s3_bucket: Option<&str>,
    replay_run_s3_key: Option<&str>,
) -> Vec<ReplayRunIndexRecord> {
    replay_runs
        .iter()
        .map(|run| {
            let replay_run_index_record_id = stable_id(
                "replay_run_index",
                &[
                    &report.research_run_report_id,
                    &run.replay_run_id,
                    replay_run_uri,
                ],
            );
            ReplayRunIndexRecord {
                replay_run_index_record_id,
                research_run_report_id: report.research_run_report_id.clone(),
                research_packet_id: report.research_packet_id.clone(),
                run_scope: report.run_scope.clone(),
                replay_run_id: run.replay_run_id.clone(),
                replay_run_uri: replay_run_uri.to_owned(),
                replay_run_s3_bucket: replay_run_s3_bucket.map(ToOwned::to_owned),
                replay_run_s3_key: replay_run_s3_key.map(ToOwned::to_owned),
                source_candidate_id: run.source_candidate_id.clone(),
                source_candidate_lifecycle_key: run.source_candidate_lifecycle_key.clone(),
                research_partition_key: run.research_partition_key.clone(),
                research_aggregate_key: run.research_aggregate_key.clone(),
                symbol_canonical: run.symbol_canonical.clone(),
                decision_available_at_ms: run.decision_available_at_ms,
                hypothesis_type: run.hypothesis_type.clone(),
                validation_adapter: run.validation_adapter.clone(),
                strategy_id_or_family: run.strategy_id_or_family.clone(),
                parameter_variant_id: run.parameter_variant_id.clone(),
                window_start_ms: run.window_start_ms,
                window_end_ms: run.window_end_ms,
                created_at_ms: report.created_at_ms,
                schema_version: REPLAY_RUN_INDEX_SCHEMA_VERSION.to_owned(),
            }
        })
        .collect()
}
