use super::*;

#[tokio::test]
async fn retest_cycle_source_state_links_manifest_and_report_for_scheduler() {
    let root = test_root("retest-source-state");
    let input = root.join("bundles.jsonl");
    let output = root.join("out");
    write_json(&input, &bundle_json());

    let summary = run(Args {
        input_bundle_file: Some(input),
        output_dir: Some(output),
        research_packet_id: "packet_state_test".to_owned(),
        run_scope: "focused_retest_local_validation".to_owned(),
        now_ms: Some(1_800_000),
        ..default_args()
    })
    .await
    .expect("research run succeeds");
    let report_file = output_file_containing(&summary, "research-run-report");
    let report = crate::io::read_research_run_report(&report_file).expect("report parses");
    let state = build_retest_cycle_source_state(
        1_900_000,
        "research-bucket",
        "research-input-manifest/schema=research_input_manifest_v1/dedupe_key=packet/manifest.json",
        "research-bucket",
        "research-run-report/schema=research_run_report_v1/dt=1970-01-01/hour=00/research_run_report_id=report/report.json",
        &report,
    );

    assert_eq!(
        state.schema_version,
        crate::model::RETEST_CYCLE_SOURCE_STATE_SCHEMA_VERSION
    );
    assert_eq!(state.research_packet_id, "packet_state_test");
    assert_eq!(state.run_scope, "focused_retest_local_validation");
    assert_eq!(state.source_candidate_ids, vec!["cand_001".to_owned()]);
    assert_eq!(state.replay_run_id_count, report.replay_run_ids.len());
    assert!(!state.safety.shadow_paper_live_enabled);
    assert_eq!(
        research_report_s3_key_from_output_files(
            "research-bucket",
            &[format!(
                "s3://research-bucket/research-run-report/schema=research_run_report_v1/dt=1970-01-01/hour=00/research_run_report_id={}/report.json",
                report.research_run_report_id
            )],
            &report.research_run_report_id,
        )
        .expect("report key is extracted"),
        format!(
            "research-run-report/schema=research_run_report_v1/dt=1970-01-01/hour=00/research_run_report_id={}/report.json",
            report.research_run_report_id
        )
    );
}
