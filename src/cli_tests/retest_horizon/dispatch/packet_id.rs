use super::*;

#[test]
fn focused_retest_dispatch_packet_id_is_stable_for_same_refresh_inputs() {
    let source_manifest: crate::model::ResearchInputManifest =
        serde_json::from_value(focused_retest_source_manifest_json())
            .expect("source manifest parses");
    let status = focused_retest_run_now_status_json();
    let mut args = default_args();
    args.input_manifest_s3_bucket = Some("research-bucket".to_owned());
    args.input_manifest_s3_key = Some(
        "research-input-manifest/schema=research_input_manifest_v1/source/manifest.json".to_owned(),
    );
    args.research_report_s3_bucket = Some("research-bucket".to_owned());
    args.research_report_s3_key =
        Some("research-run-report/schema=research_run_report_v1/report.json".to_owned());
    args.run_scope = "focused_retest_local_validation".to_owned();

    let build_a = crate::focused_retest::build_focused_retest_manifest(
        &status,
        &source_manifest,
        &crate::focused_retest::FocusedRetestBuildOptions {
            generated_at_ms: 7_400_000,
            research_packet_id: "research_focus_7400000".to_owned(),
            run_scope: "focused_retest_local_validation".to_owned(),
            next_actions: crate::focused_retest::default_focused_retest_actions(),
            candidate_lifecycle_key_filter: Vec::new(),
            historical_replay_index_ref_mode:
                crate::focused_retest::HistoricalReplayIndexRefMode::Auto,
            s3_write: true,
        },
    )
    .expect("focused build a succeeds");
    let build_b = crate::focused_retest::build_focused_retest_manifest(
        &status,
        &source_manifest,
        &crate::focused_retest::FocusedRetestBuildOptions {
            generated_at_ms: 7_500_000,
            research_packet_id: "research_focus_7500000".to_owned(),
            run_scope: "focused_retest_local_validation".to_owned(),
            next_actions: crate::focused_retest::default_focused_retest_actions(),
            candidate_lifecycle_key_filter: Vec::new(),
            historical_replay_index_ref_mode:
                crate::focused_retest::HistoricalReplayIndexRefMode::Auto,
            s3_write: true,
        },
    )
    .expect("focused build b succeeds");

    let first_id = focused_retest_dispatch_packet_id(&args, Some(7_201_300), &build_a)
        .expect("first dispatch id");
    let second_id = focused_retest_dispatch_packet_id(&args, Some(7_201_300), &build_b)
        .expect("second dispatch id");
    let advanced_l1_id = focused_retest_dispatch_packet_id(&args, Some(7_801_300), &build_b)
        .expect("advanced l1 dispatch id");

    assert_eq!(first_id, second_id);
    assert_ne!(first_id, advanced_l1_id);
    assert!(first_id.starts_with("research_focus_"));
    assert_eq!(
        focused_retest_dispatch_manifest_s3_key(&first_id)
            .expect("dispatch key")
            .as_str(),
        format!(
            "research-input-manifest/schema=research_input_manifest_v1/dedupe_key={first_id}/manifest.json"
        )
    );
}
