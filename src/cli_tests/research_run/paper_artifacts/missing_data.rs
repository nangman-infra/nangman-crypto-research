use super::support::{paper_artifact_run_args, read_report};
use super::*;

#[tokio::test]
async fn data_missing_retest_does_not_create_paper_watch() {
    let root = test_root("paper-watch-data-missing");
    let input = root.join("bundles.json");
    let output = root.join("out");

    write_json(
        &input,
        &Value::Array(vec![bundle_json_with_gate_inputs(8, 1_300)]),
    );

    let summary = run(paper_artifact_run_args(
        input,
        None,
        None,
        Vec::new(),
        output,
    ))
    .await
    .expect("research run succeeds");

    let report = read_report(&summary);
    assert_eq!(report["summary_findings"][0]["bias"], json!("RETEST_BIAS"));
    assert_eq!(report["paper_watch_candidates"], json!([]));
    assert!(
        summary
            .output_files
            .iter()
            .all(|path| !path.contains("/paper-watch-candidate/"))
    );
}
