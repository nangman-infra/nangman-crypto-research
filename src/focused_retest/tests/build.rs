use super::fixtures::{source_manifest, status_with_focus_rows};
use crate::focused_retest::{
    FocusedRetestBuildOptions, HistoricalReplayIndexRefMode, build_focused_retest_manifest,
};

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
