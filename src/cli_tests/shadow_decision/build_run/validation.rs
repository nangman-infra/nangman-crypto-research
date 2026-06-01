use super::super::*;

#[tokio::test]
async fn build_shadow_cycle_decision_rejects_empty_shadow_runs() {
    let root = test_root("shadow-decision-build-empty");
    let shadow_file = root.join("shadow-runs.json");
    let output_file = root.join("shadow-cycle-decision.json");
    write_json(&shadow_file, &json!([]));

    let args = parse_args(
        [
            "--build-shadow-cycle-decision".to_owned(),
            "--shadow-validation-run-file".to_owned(),
            shadow_file.display().to_string(),
            "--shadow-cycle-decision-output-file".to_owned(),
            output_file.display().to_string(),
        ]
        .into_iter(),
    )
    .expect("build args parse")
    .expect("build args returned");

    let error = run(args)
        .await
        .expect_err("empty shadow validation input is rejected");
    assert!(
        error
            .to_string()
            .contains("at least one shadow validation run")
    );
}
