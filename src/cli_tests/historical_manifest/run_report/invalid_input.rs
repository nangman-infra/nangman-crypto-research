use super::*;

#[tokio::test]
async fn lookahead_mismatch_is_invalid_input() {
    let root = test_root("lookahead");
    let input = root.join("bundles.jsonl");
    let output = root.join("out");
    let mut bundle = bundle_json();
    bundle["forbidden_lookahead_boundary_ms"] = json!(1_299);
    write_json(&input, &bundle);

    let summary = run(Args {
        input_bundle_file: Some(input),
        output_dir: Some(output),
        research_packet_id: "packet_test".to_owned(),
        run_scope: "test".to_owned(),
        now_ms: Some(1_800_000),
        ..default_args()
    })
    .await
    .expect("run succeeds with partial report");

    let report_text = fs::read_to_string(&summary.output_files[0]).expect("report exists");
    assert!(report_text.contains("invalid_input"));
    assert!(report_text.contains("lookahead_boundary_mismatch"));
}
