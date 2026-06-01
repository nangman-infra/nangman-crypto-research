use super::support::{
    first_registry_record, output_line_count, paper_artifact_run_args, read_report,
    write_gate_inputs,
};
use super::*;

#[tokio::test]
async fn completed_shadow_validation_input_creates_paper_artifacts_without_live_approval() {
    let root = test_root("paper-from-shadow");
    let shadow_output = root.join("shadow-out");
    let paper_output = root.join("paper-out");
    let completed_shadow_file = root.join("completed-shadow.json");
    let gate_inputs = write_gate_inputs(&root, 31);

    let shadow_summary = run(paper_artifact_run_args(
        gate_inputs.input.clone(),
        Some(gate_inputs.delta.clone()),
        Some(gate_inputs.regime.clone()),
        Vec::new(),
        shadow_output,
    ))
    .await
    .expect("shadow run succeeds");

    let shadow_output_file = output_file_containing(&shadow_summary, "/shadow-validation-run/");
    let completed_shadow_runs = fs::read_to_string(&shadow_output_file)
        .expect("shadow output exists")
        .lines()
        .map(|line| {
            let mut run: Value = serde_json::from_str(line).expect("shadow line parses");
            run["status"] = json!("completed");
            run["passed"] = json!(true);
            run["paper_trade_candidate_contract_version"] = json!("paper_trade_candidate_v1");
            run
        })
        .collect::<Vec<_>>();
    write_json(&completed_shadow_file, &Value::Array(completed_shadow_runs));

    let summary = run(paper_artifact_run_args(
        gate_inputs.input,
        Some(gate_inputs.delta),
        Some(gate_inputs.regime),
        vec![completed_shadow_file],
        paper_output,
    ))
    .await
    .expect("paper run succeeds");

    assert_eq!(summary.shadow_validation_runs_loaded, 31);
    assert_eq!(summary.shadow_validation_runs_created, 0);
    assert_eq!(summary.paper_trade_candidates_created, 31);
    assert_eq!(summary.paper_trade_runs_created, 31);
    assert_eq!(summary.paper_trade_summaries_created, 31);
    assert_eq!(summary.paper_trade_marks_created, 31);

    let report = read_report(&summary);
    assert_eq!(
        report["summary_findings"][0]["bias"],
        json!("PROMOTE_TO_PAPER_BIAS")
    );
    assert_eq!(
        report["paper_trade_candidates"]
            .as_array()
            .expect("paper candidate ids")
            .len(),
        31
    );
    for marker in [
        "/paper-trade-candidate/",
        "/paper-trade-run/",
        "/paper-trade-summary/",
        "/paper-trade-mark/",
    ] {
        assert_eq!(output_line_count(&summary, marker), 31);
    }
    let registry = first_registry_record(&summary);
    assert_eq!(
        registry["current_research_stage"],
        json!("paper_candidate_bias")
    );
    let report_text = serde_json::to_string(&report).expect("report serializes");
    assert!(!report_text.contains("EXECUTION_APPROVED"));
    assert!(!report_text.contains("LIVE_READY"));
}
