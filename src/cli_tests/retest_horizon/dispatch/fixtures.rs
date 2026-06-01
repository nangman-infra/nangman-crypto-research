use super::*;

pub(super) fn retest_cycle_source_state() -> RetestCycleSourceState {
    RetestCycleSourceState {
        schema_version: RETEST_CYCLE_SOURCE_STATE_SCHEMA_VERSION.to_owned(),
        generated_at_ms: 7_300_000,
        research_packet_id: "source_packet".to_owned(),
        run_scope: "focused_retest_local_validation".to_owned(),
        source_manifest_s3_bucket: "research-bucket".to_owned(),
        source_manifest_s3_key:
            "research-input-manifest/schema=research_input_manifest_v1/source/manifest.json"
                .to_owned(),
        source_research_report_s3_bucket: "research-bucket".to_owned(),
        source_research_report_s3_key:
            "research-run-report/schema=research_run_report_v1/report.json".to_owned(),
        source_research_report_id: "research_report_source".to_owned(),
        source_candidate_ids: vec!["cand_focus".to_owned()],
        replay_run_id_count: 1,
        summary_findings_count: 1,
        shadow_validation_run_count: 0,
        paper_trade_candidate_count: 0,
        safety: RetestCycleSourceStateSafety {
            dispatcher_prefix: "research-input-manifest/".to_owned(),
            state_s3_write: true,
            ecs_task_started: false,
            shadow_paper_live_enabled: false,
        },
    }
}
