use super::super::*;

#[tokio::test]
async fn build_shadow_cycle_decision_writes_partitioned_output_dir() {
    let root = test_root("shadow-decision-build-output-dir");
    let shadow_file = root.join("shadow-runs.json");
    let output_dir = root.join("outputs");
    let decision_ms = 1_780_000_000_000_i64;
    let materialized_target_ms = decision_ms + DAY_MS;
    write_json(
        &shadow_file,
        &json!([shadow_validation_run_json(
            "shadow_a",
            "cand_a",
            "XAUT",
            decision_ms,
            30
        )]),
    );

    let args = parse_args(
        [
            "--build-shadow-cycle-decision".to_owned(),
            "--shadow-validation-run-file".to_owned(),
            shadow_file.display().to_string(),
            "--shadow-cycle-latest-l1-as-of-ms".to_owned(),
            materialized_target_ms.to_string(),
            "--output-dir".to_owned(),
            output_dir.display().to_string(),
            "--now-ms".to_owned(),
            "1780100000000".to_owned(),
        ]
        .into_iter(),
    )
    .expect("build args parse")
    .expect("build args returned");

    let summary = run(args).await.expect("shadow cycle decision builds");
    assert_eq!(summary.shadow_cycle_decisions_created, 1);
    assert_eq!(summary.output_files.len(), 1);

    let output_file = PathBuf::from(&summary.output_files[0]);
    assert!(output_file.starts_with(&output_dir));
    assert!(
        output_file
            .display()
            .to_string()
            .contains("shadow-cycle-decision/schema=research_shadow_cycle_decision_v1")
    );
    assert!(output_file.exists());
}
