use super::super::*;

pub(super) fn assert_shadow_output(summary: &RunSummary) {
    assert_eq!(summary.shadow_validation_runs_created, 31);

    let shadow_output_file = output_file_containing(summary, "/shadow-validation-run/");
    let shadow_output_text =
        fs::read_to_string(&shadow_output_file).expect("shadow validation output exists");

    assert_eq!(shadow_output_text.lines().count(), 31);
    assert!(!shadow_output_text.contains("EXECUTION_APPROVED"));
    assert!(!shadow_output_text.contains("LIVE_READY"));
}

pub(super) fn assert_report(summary: &RunSummary) {
    let report = read_report(summary);

    assert_eq!(
        report["partition_aggregates"][0]["gate_bias"],
        json!("PROMOTE_TO_SHADOW_BIAS")
    );
    assert_eq!(report["paper_trade_candidates"], json!([]));
    assert_eq!(
        report["research_gate_policy"]["allow_promote_to_paper_bias"],
        json!(false)
    );
    assert_eq!(
        report["partition_aggregates"][0]["train_validation_split_summary"]["passed"],
        json!(true)
    );
    assert_eq!(
        report["partition_aggregates"][0]["cost_stressed_mean_net_after_cost_bps"],
        json!(16.0)
    );
    assert_eq!(
        report["partition_aggregates"][0]["gate_reason_codes"],
        json!(["deterministic_shadow_gate_passed"])
    );
    assert_eq!(
        report["partition_aggregates"][0]["completed_count"],
        json!(31)
    );
    assert_eq!(
        report["partition_aggregates"][0]["inferred_unseen_window_count"],
        json!(30)
    );
    assert_eq!(
        report["shadow_validation_runs"]
            .as_array()
            .expect("shadow run ids are present")
            .len(),
        31
    );
    assert_eq!(
        report["shadow_validation_runs"][0]["schema_version"],
        json!("shadow_validation_run_v1")
    );
    assert_eq!(
        report["shadow_validation_runs"][0]["watch_window_policy"]["mode"],
        json!("forward_observation_only")
    );
    assert_eq!(
        report["shadow_validation_runs"][0]["termination_policy"]["no_order_execution"],
        json!(true)
    );

    let report_text = serde_json::to_string(&report).expect("report serializes");
    assert!(!report_text.contains("EXECUTION_APPROVED"));
    assert!(!report_text.contains("LIVE_READY"));
}

pub(super) fn assert_registry(summary: &RunSummary) {
    let registry_file = output_file_containing(summary, "/research-aggregate-registry/");
    let registry_text = fs::read_to_string(&registry_file).expect("registry output exists");
    let registry: Value = serde_json::from_str(
        registry_text
            .lines()
            .next()
            .expect("registry output has one line"),
    )
    .expect("registry line parses");

    assert_eq!(
        registry["current_research_stage"],
        json!("shadow_candidate")
    );
    assert_eq!(registry["gate_bias"], json!("PROMOTE_TO_SHADOW_BIAS"));
    assert_eq!(
        registry["linked_shadow_validation_run_ids"]
            .as_array()
            .expect("shadow validation ids are recorded")
            .len(),
        31
    );
}

fn read_report(summary: &RunSummary) -> Value {
    serde_json::from_str(&fs::read_to_string(&summary.output_files[0]).expect("report exists"))
        .expect("report json parses")
}
