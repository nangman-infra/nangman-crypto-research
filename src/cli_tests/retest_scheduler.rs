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

#[tokio::test]
async fn build_focused_retest_manifest_from_status_and_source_manifest() {
    let root = test_root("focused-retest-manifest-cli");
    let status_file = root.join("retest-horizon-status.json");
    let source_manifest_file = root.join("research-input-manifest.json");
    let output_file = root.join("focused-retest-manifest.json");
    let summary_file = root.join("focused-retest-manifest.summary.json");
    write_json(&status_file, &focused_retest_status_json());
    write_json(
        &source_manifest_file,
        &focused_retest_source_manifest_json(),
    );

    let args = parse_args(
        [
            "--build-focused-retest-manifest".to_owned(),
            "--retest-horizon-status-file".to_owned(),
            status_file.display().to_string(),
            "--input-manifest-file".to_owned(),
            source_manifest_file.display().to_string(),
            "--focused-retest-manifest-output-file".to_owned(),
            output_file.display().to_string(),
            "--focused-retest-summary-output-file".to_owned(),
            summary_file.display().to_string(),
            "--research-packet-id".to_owned(),
            "research_focus_test".to_owned(),
            "--run-scope".to_owned(),
            "focused_retest_local_validation".to_owned(),
            "--now-ms".to_owned(),
            "1779719361452".to_owned(),
        ]
        .into_iter(),
    )
    .expect("focused args parse")
    .expect("focused args returned");
    let summary = run(args).await.expect("focused manifest builds");

    assert_eq!(summary.retest_horizon_statuses_validated, 1);
    assert_eq!(summary.focused_retest_manifests_created, 1);
    assert_eq!(summary.focused_retest_horizon_count, 1);
    assert_eq!(summary.focused_retest_candidate_bundle_refs, 1);
    assert_eq!(summary.output_files.len(), 2);

    let manifest: Value =
        serde_json::from_slice(&fs::read(&output_file).expect("manifest")).expect("manifest json");
    assert_eq!(manifest["research_packet_id"], json!("research_focus_test"));
    assert_eq!(
        manifest["candidate_bundle_refs"].as_array().unwrap().len(),
        1
    );
    assert_eq!(
        manifest["candidate_bundle_refs"][0]["uri"],
        json!(
            "s3://bucket/candidate-evidence-bundle/priority=p0/candidate_id=cand_focus/part-000001.jsonl"
        )
    );
    assert_eq!(
        manifest["historical_replay_run_index_refs"]
            .as_array()
            .unwrap()
            .len(),
        1
    );

    let focus_summary: Value =
        serde_json::from_slice(&fs::read(&summary_file).expect("summary")).expect("summary json");
    assert_eq!(
        focus_summary["schema_version"],
        json!("research_focused_retest_manifest_summary_v1")
    );
    assert_eq!(
        focus_summary["focused"]["selected_candidate_bundle_ref_count"],
        json!(1)
    );
    assert_eq!(focus_summary["safety"]["s3_write"], json!(false));
}

#[tokio::test]
async fn build_focused_retest_manifest_rejects_empty_selection() {
    let root = test_root("focused-retest-empty-cli");
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
            "--build-focused-retest-manifest".to_owned(),
            "--retest-horizon-status-file".to_owned(),
            status_file.display().to_string(),
            "--input-manifest-file".to_owned(),
            source_manifest_file.display().to_string(),
            "--focused-retest-manifest-output-file".to_owned(),
            output_file.display().to_string(),
            "--focused-retest-next-actions".to_owned(),
            "run_research_replay_for_horizon".to_owned(),
        ]
        .into_iter(),
    )
    .expect("focused args parse")
    .expect("focused args returned");
    let error = run(args)
        .await
        .expect_err("empty focused selection is rejected");

    assert!(
        error
            .to_string()
            .contains("selected zero candidate bundle refs")
    );
    assert!(!output_file.exists());
}
