use super::*;

#[test]
fn output_partition_uses_execution_time_without_rewriting_report_time() {
    let root = test_root("output-partition-time");
    let bundles = vec![
        serde_json::from_value(bundle_json()).expect("candidate bundle test json matches model"),
    ];
    let report =
        crate::report::build_report("packet_test", "test", 7_200_000, &bundles, &[], &[], &[]);

    let output_artifacts = crate::io::ResearchOutputArtifacts {
        report: &report,
        replay_runs: &[],
        shadow_validation_runs: &[],
        paper_watch_candidates: &[],
        paper_trade_candidates: &[],
        paper_trade_runs: &[],
        paper_trade_summaries: &[],
        paper_trade_marks: &[],
        output_partition_at_ms: 3_600_000,
    };
    let written = crate::io::write_research_outputs(&root, &output_artifacts).expect("write ok");

    let relative = written[0]
        .strip_prefix(&root)
        .expect("output is under test root")
        .display()
        .to_string();
    assert!(
        relative.contains("dt=1970-01-01/hour=01"),
        "output partition should use execution time, got {relative}"
    );
    let report_json: Value =
        serde_json::from_str(&fs::read_to_string(&written[0]).expect("report exists"))
            .expect("report json parses");
    assert_eq!(report_json["created_at_ms"], json!(7_200_000));
}
