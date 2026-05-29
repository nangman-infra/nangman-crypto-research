use super::*;
use crate::model::{ResearchArtifactRef, ResearchInputManifest, ResearchRuntimeBudgetPolicy};
use serde_json::{Value, json};

#[test]
fn builds_focused_manifest_for_ready_actions() {
    let source_manifest = source_manifest();
    let status = status_with_focus_rows();
    let build = build_focused_retest_manifest(
        &status,
        &source_manifest,
        &FocusedRetestBuildOptions {
            generated_at_ms: 1_779_719_361_452,
            research_packet_id: "research_focus_test".to_owned(),
            run_scope: "focused_retest_local_validation".to_owned(),
            next_actions: vec!["accumulate_completed_native_replay_samples".to_owned()],
            candidate_lifecycle_key_filter: Vec::new(),
            historical_replay_index_ref_mode: HistoricalReplayIndexRefMode::Auto,
            s3_write: false,
        },
    )
    .expect("focused manifest builds");

    assert_eq!(
        build.manifest.research_packet_id.as_deref(),
        Some("research_focus_test")
    );
    assert_eq!(build.manifest.candidate_bundle_refs.len(), 1);
    assert_eq!(
        build.manifest.candidate_bundle_refs[0].uri,
        "s3://bucket/candidate-evidence-bundle/priority=p0/candidate_id=cand_a/part-000001.jsonl"
    );
    assert_eq!(build.manifest.historical_replay_run_index_refs.len(), 1);
    assert_eq!(
        build
            .manifest
            .runtime_budget_policy
            .max_candidate_bundle_count,
        1
    );
    assert_eq!(build.summary.focused.focus_horizon_count, 1);
    assert_eq!(build.summary.focused.selected_candidate_bundle_ref_count, 1);
}

#[test]
fn rejects_empty_focused_manifest() {
    let source_manifest = source_manifest();
    let status = status_with_focus_rows();
    let error = build_focused_retest_manifest(
        &status,
        &source_manifest,
        &FocusedRetestBuildOptions {
            generated_at_ms: 1_779_719_361_452,
            research_packet_id: "research_focus_test".to_owned(),
            run_scope: "focused_retest_local_validation".to_owned(),
            next_actions: vec!["run_research_replay_for_horizon".to_owned()],
            candidate_lifecycle_key_filter: Vec::new(),
            historical_replay_index_ref_mode: HistoricalReplayIndexRefMode::Auto,
            s3_write: false,
        },
    )
    .expect_err("empty selected refs are rejected");

    assert!(
        error
            .to_string()
            .contains("selected zero candidate bundle refs")
    );
}

#[test]
fn filters_focused_manifest_by_candidate_lifecycle_key() {
    let source_manifest = source_manifest();
    let status = status_with_focus_rows();
    let build = build_focused_retest_manifest(
        &status,
        &source_manifest,
        &FocusedRetestBuildOptions {
            generated_at_ms: 1_779_719_361_452,
            research_packet_id: "research_focus_test".to_owned(),
            run_scope: "shadow_sample_accumulation_local_validation".to_owned(),
            next_actions: vec!["accumulate_completed_native_replay_samples".to_owned()],
            candidate_lifecycle_key_filter: vec!["cand_a:v1".to_owned()],
            historical_replay_index_ref_mode: HistoricalReplayIndexRefMode::Auto,
            s3_write: false,
        },
    )
    .expect("filtered focused manifest builds");

    assert_eq!(build.summary.focused.focus_candidate_count, 1);
    assert_eq!(
        build.summary.focused.rows[0]
            .candidate_lifecycle_key
            .as_deref(),
        Some("cand_a:v1")
    );
    assert_eq!(build.manifest.candidate_bundle_refs.len(), 1);
}

fn source_manifest() -> ResearchInputManifest {
    ResearchInputManifest {
        schema_version: "research_input_manifest_v1".to_owned(),
        research_packet_id: Some("source_packet".to_owned()),
        run_scope: Some("current_approved".to_owned()),
        candidate_bundle_refs: vec![
            ResearchArtifactRef {
                uri: "s3://bucket/candidate-evidence-bundle/priority=p0/candidate_id=cand_a/part-000001.jsonl".to_owned(),
            },
            ResearchArtifactRef {
                uri: "s3://bucket/candidate-evidence-bundle/priority=p0/candidate_id=cand_b/part-000001.jsonl".to_owned(),
            },
        ],
        market_feature_delta_refs: Vec::new(),
        market_regime_context_refs: Vec::new(),
        shadow_validation_run_refs: Vec::new(),
        hypothesis_harness_result_refs: Vec::new(),
        oss_adapter_run_refs: Vec::new(),
        historical_replay_run_refs: Vec::new(),
        historical_replay_run_index_refs: vec![ResearchArtifactRef {
            uri: "s3://research/replay-run-index/part-000001.jsonl".to_owned(),
        }],
        runtime_budget_policy: ResearchRuntimeBudgetPolicy::default(),
    }
}

fn status_with_focus_rows() -> Value {
    json!({
        "schema_version": "research_horizon_status_checkpoint_v1",
        "safety": {
            "s3_write": false,
            "ecs_task_started": false,
            "dispatcher_mode_changed": false,
            "local_summary_only": true,
            "shadow_paper_live_enabled": false
        },
        "stage_state": {
            "candidate_generated": true,
            "research_replay_completed": true,
            "promotion_passed": false,
            "shadow_created": false,
            "paper_created": false,
            "live_enabled": false
        },
        "next_decision": {
            "verdict": "WAIT_FOR_MARKET_L1_HORIZON",
            "scheduler_hint": {
                "latest_l1_as_of_ms": 1_779_710_400_000_i64,
                "run_research_after_l1_as_of_ms": 1_779_719_361_452_i64,
                "run_now_replay_ready": false,
                "promotion_ready_for_review": false
            },
            "blocked_actions": [
                "do_not_create_shadow_without_promotion",
                "do_not_create_paper_without_passed_shadow",
                "do_not_enable_live_from_research_batch"
            ]
        },
        "by_symbol": [
            {
                "symbol": "AAVE",
                "candidates": [
                    {
                        "candidate_id": "cand_a",
                        "candidate_lifecycle_key": "cand_a:v1",
                        "hypothesis_type": "event_reaction",
                        "research_priority": "p0",
                        "horizons": [
                            {
                                "horizon": "1h",
                                "next_action": "accumulate_completed_native_replay_samples",
                                "symbols": ["AAVE"],
                                "replay_run_count": 3,
                                "completed_count": 1,
                                "completed_sample_deficit": 2,
                                "reason_codes": ["sample_deficit"]
                            }
                        ]
                    }
                ]
            }
        ]
    })
}
