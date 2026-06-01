use super::*;

#[tokio::test]
async fn retest_cycle_scheduler_waits_before_not_before() {
    let root = test_root("retest-cycle-scheduler-wait");
    let status_file = root.join("retest-horizon-status.json");
    let source_manifest_file = root.join("research-input-manifest.json");
    let output_file = root.join("focused-retest-manifest.json");
    write_json(&status_file, &focused_retest_status_json());
    write_json(
        &source_manifest_file,
        &focused_retest_source_manifest_json(),
    );

    let args = parse_args(
        [
            "--run-retest-cycle-scheduler".to_owned(),
            "--retest-horizon-status-file".to_owned(),
            status_file.display().to_string(),
            "--input-manifest-file".to_owned(),
            source_manifest_file.display().to_string(),
            "--focused-retest-manifest-output-file".to_owned(),
            output_file.display().to_string(),
            "--now-ms".to_owned(),
            "1779719361451".to_owned(),
        ]
        .into_iter(),
    )
    .expect("scheduler args parse")
    .expect("scheduler args returned");
    let summary = run(args).await.expect("scheduler waits");

    assert_eq!(summary.retest_horizon_statuses_validated, 1);
    assert_eq!(
        summary.retest_cycle_scheduler_action,
        Some("WAIT_UNTIL_MARKET_L1_HORIZON_MATERIALIZES".to_owned())
    );
    assert_eq!(
        summary.retest_cycle_run_not_before_ms,
        Some(1_779_719_361_452)
    );
    assert_eq!(summary.focused_retest_manifests_created, 0);
    assert!(summary.output_files.is_empty());
    assert!(!output_file.exists());
}

#[tokio::test]
async fn retest_cycle_scheduler_requires_fresh_status_after_wait_deadline() {
    let root = test_root("retest-cycle-scheduler-refresh");
    let status_file = root.join("retest-horizon-status.json");
    let source_manifest_file = root.join("research-input-manifest.json");
    let output_file = root.join("focused-retest-manifest.json");
    write_json(&status_file, &focused_retest_status_json());
    write_json(
        &source_manifest_file,
        &focused_retest_source_manifest_json(),
    );

    let args = parse_args(
        [
            "--run-retest-cycle-scheduler".to_owned(),
            "--retest-horizon-status-file".to_owned(),
            status_file.display().to_string(),
            "--input-manifest-file".to_owned(),
            source_manifest_file.display().to_string(),
            "--focused-retest-manifest-output-file".to_owned(),
            output_file.display().to_string(),
            "--now-ms".to_owned(),
            "1779719361452".to_owned(),
        ]
        .into_iter(),
    )
    .expect("scheduler args parse")
    .expect("scheduler args returned");
    let summary = run(args).await.expect("scheduler asks for refresh");

    assert_eq!(
        summary.retest_cycle_scheduler_action,
        Some("REFRESH_RETEST_HORIZON_STATUS_AFTER_WAIT_DEADLINE".to_owned())
    );
    assert_eq!(summary.focused_retest_manifests_created, 0);
    assert!(summary.output_files.is_empty());
    assert!(!output_file.exists());
}

#[tokio::test]
async fn retest_cycle_scheduler_builds_focused_manifest_when_run_now() {
    let root = test_root("retest-cycle-scheduler-run-now");
    let status_file = root.join("retest-horizon-status.json");
    let source_manifest_file = root.join("research-input-manifest.json");
    let output_file = root.join("focused-retest-manifest.json");
    write_json(&status_file, &focused_retest_run_now_status_json());
    write_json(
        &source_manifest_file,
        &focused_retest_source_manifest_json(),
    );

    let args = parse_args(
        [
            "--run-retest-cycle-scheduler".to_owned(),
            "--retest-horizon-status-file".to_owned(),
            status_file.display().to_string(),
            "--input-manifest-file".to_owned(),
            source_manifest_file.display().to_string(),
            "--focused-retest-manifest-output-file".to_owned(),
            output_file.display().to_string(),
            "--research-packet-id".to_owned(),
            "research_focus_scheduler_test".to_owned(),
            "--now-ms".to_owned(),
            "1779719361452".to_owned(),
        ]
        .into_iter(),
    )
    .expect("scheduler args parse")
    .expect("scheduler args returned");
    let summary = run(args).await.expect("scheduler builds focused manifest");

    assert_eq!(
        summary.retest_cycle_scheduler_action,
        Some("RUN_FOCUSED_RETEST_RESEARCH".to_owned())
    );
    assert_eq!(summary.focused_retest_manifests_created, 1);
    assert_eq!(summary.focused_retest_candidate_bundle_refs, 1);
    assert_eq!(summary.output_files.len(), 2);
    assert!(output_file.exists());
}
