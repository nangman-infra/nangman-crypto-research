use super::*;
use crate::model::{
    ResearchArtifactRef, ResearchInputManifest, ResearchRuntimeBudgetPolicy,
    RetestCycleSourceState, RetestCycleSourceStateSafety,
};
use serde_json::json;

#[test]
fn research_input_manifest_key_rejects_slash_in_packet_id() {
    let mut manifest = manifest("packet/unsafe");

    let error = research_input_manifest_key("", &manifest, 1_779_935_219_010)
        .expect_err("slash in generated key component should be rejected");

    assert!(error.to_string().contains("single safe S3 key segment"));
    manifest.research_packet_id = Some("packet_safe".to_owned());
    assert!(
        research_input_manifest_key("", &manifest, 1_779_935_219_010)
            .expect("safe packet id should build")
            .contains("research_packet_id=packet_safe")
    );
}

#[test]
fn retest_cycle_source_state_key_rejects_slash_in_report_id() {
    let mut state = source_state();
    state.source_research_report_id = "report/unsafe".to_owned();

    let error = retest_cycle_source_state_key("", &state, 1_779_935_219_010)
        .expect_err("slash in report id should be rejected");

    assert!(error.to_string().contains("single safe S3 key segment"));
}

#[test]
fn retest_horizon_keys_use_default_prefixes() {
    let plan_key = retest_horizon_plan_key("", &json!({"generated_at_ms": 42}), 0)
        .expect("plan key should build");
    let status_key = retest_horizon_status_key("", &json!({"generated_at_ms": 42}), 0)
        .expect("status key should build");

    assert!(plan_key.starts_with("retest-horizon-plan/"));
    assert!(status_key.starts_with("retest-horizon-status/"));
}

fn manifest(packet_id: &str) -> ResearchInputManifest {
    ResearchInputManifest {
        schema_version: "research_input_manifest_v1".to_owned(),
        research_packet_id: Some(packet_id.to_owned()),
        run_scope: Some("test".to_owned()),
        candidate_bundle_refs: Vec::new(),
        market_feature_delta_refs: Vec::new(),
        market_regime_context_refs: Vec::new(),
        shadow_validation_run_refs: Vec::new(),
        hypothesis_harness_result_refs: Vec::new(),
        oss_adapter_run_refs: Vec::new(),
        historical_replay_run_refs: vec![ResearchArtifactRef {
            uri: "s3://bucket/replay.jsonl".to_owned(),
        }],
        historical_replay_run_index_refs: Vec::new(),
        runtime_budget_policy: ResearchRuntimeBudgetPolicy::default(),
    }
}

fn source_state() -> RetestCycleSourceState {
    RetestCycleSourceState {
        schema_version: "research_retest_cycle_source_state_v1".to_owned(),
        generated_at_ms: 1,
        research_packet_id: "packet_safe".to_owned(),
        run_scope: "test".to_owned(),
        source_manifest_s3_bucket: "bucket".to_owned(),
        source_manifest_s3_key: "research-input-manifest/manifest.json".to_owned(),
        source_research_report_s3_bucket: "bucket".to_owned(),
        source_research_report_s3_key: "research-run-report/report.json".to_owned(),
        source_research_report_id: "report_safe".to_owned(),
        source_candidate_ids: Vec::new(),
        replay_run_id_count: 0,
        summary_findings_count: 0,
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
