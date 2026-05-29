use super::*;

#[tokio::test]
async fn retest_horizon_status_file_validates_without_research_inputs() {
    let root = test_root("retest-status-cli");
    let status_file = root.join("retest-horizon-status.json");
    write_json(&status_file, &retest_horizon_wait_status_json());

    let args = parse_args(
        [
            "--retest-horizon-status-file".to_owned(),
            status_file.display().to_string(),
        ]
        .into_iter(),
    )
    .expect("status args parse")
    .expect("status args returned");
    let summary = run(args).await.expect("status validates");

    assert_eq!(summary.retest_horizon_statuses_validated, 1);
    assert_eq!(
        summary.retest_cycle_scheduler_action,
        Some("WAIT_UNTIL_MARKET_L1_HORIZON_MATERIALIZES".to_owned())
    );
    assert_eq!(
        summary.retest_cycle_run_not_before_ms,
        Some(1_779_719_361_452)
    );
    assert_eq!(summary.processed_bundles, 0);
    assert!(summary.output_files.is_empty());
}

#[tokio::test]
async fn retest_horizon_status_file_rejects_live_enabled() {
    let root = test_root("retest-status-unsafe-cli");
    let status_file = root.join("retest-horizon-status.json");
    let mut status = retest_horizon_wait_status_json();
    status["stage_state"]["live_enabled"] = json!(true);
    write_json(&status_file, &status);

    let args = parse_args(
        [
            "--retest-horizon-status-file".to_owned(),
            status_file.display().to_string(),
        ]
        .into_iter(),
    )
    .expect("status args parse")
    .expect("status args returned");
    let error = run(args)
        .await
        .expect_err("unsafe retest status is rejected");

    assert!(error.to_string().contains("live trading"));
}

#[tokio::test]
async fn build_retest_horizon_status_from_plan_file() {
    let root = test_root("retest-status-build-cli");
    let plan_file = root.join("retest-horizon-plan.json");
    let output_file = root.join("retest-horizon-status.json");
    write_json(&plan_file, &retest_horizon_plan_json());

    let args = parse_args(
        [
            "--build-retest-horizon-status".to_owned(),
            "--retest-horizon-plan-file".to_owned(),
            plan_file.display().to_string(),
            "--retest-horizon-status-output-file".to_owned(),
            output_file.display().to_string(),
            "--now-ms".to_owned(),
            "1779714000000".to_owned(),
        ]
        .into_iter(),
    )
    .expect("status build args parse")
    .expect("status build args returned");
    let summary = run(args).await.expect("status builds");

    assert_eq!(summary.retest_horizon_statuses_validated, 1);
    assert_eq!(
        summary.retest_cycle_scheduler_action,
        Some("RUN_FOCUSED_RETEST_RESEARCH".to_owned())
    );
    assert_eq!(summary.retest_cycle_run_not_before_ms, None);
    assert_eq!(
        summary.output_files,
        vec![output_file.display().to_string()]
    );

    let status: Value =
        serde_json::from_slice(&fs::read(&output_file).expect("status")).expect("status json");
    assert_eq!(
        status["schema_version"],
        json!("research_horizon_status_checkpoint_v1")
    );
    assert_eq!(status["safety"]["checkpoint_s3_write"], json!(false));
    assert_eq!(status["selected_symbols"], json!(["AAVE"]));
    assert_eq!(
        status["by_symbol"][0]["candidates"][1]["horizons"][0]["next_action"],
        json!("wait_for_market_l1_horizon")
    );
}

#[tokio::test]
async fn build_retest_horizon_plan_from_manifest_and_report() {
    let root = test_root("retest-plan-build-cli");
    let bundle = root.join("bundle.json");
    let delta = root.join("delta.json");
    let regime = root.join("regime.json");
    let manifest = root.join("manifest.json");
    let research_output = root.join("research-out");
    let plan_output = root.join("retest-horizon-plan.json");

    write_json(&bundle, &bundle_json_with_gate_inputs(1, 1_300));
    write_json(
        &delta,
        &json!([market_delta_json("delta_001", 1_300, 3_601_300, 0.021)]),
    );
    write_json(
        &regime,
        &json!([market_regime_json("regime_001", 1_300, 3_601_300)]),
    );
    write_json(
        &manifest,
        &json!({
            "schema_version": "research_input_manifest_v1",
            "research_packet_id": "manifest_packet",
            "run_scope": "manifest_batch",
            "candidate_bundle_refs": [{ "uri": bundle.display().to_string() }],
            "market_feature_delta_refs": [{ "uri": delta.display().to_string() }],
            "market_regime_context_refs": [{ "uri": regime.display().to_string() }],
            "runtime_budget_policy": {
                "max_candidate_bundle_count": 10,
                "max_market_artifact_ref_count": 10,
                "max_historical_replay_run_ref_count": 10,
                "max_replay_run_count": 20
            }
        }),
    );

    let research_summary = run(Args {
        input_manifest_file: Some(manifest.clone()),
        output_dir: Some(research_output),
        now_ms: Some(7_300_000),
        ..default_args()
    })
    .await
    .expect("research report builds");
    let report_file = output_file_containing(&research_summary, "research-run-report");

    let args = parse_args(
        [
            "--build-retest-horizon-plan".to_owned(),
            "--input-manifest-file".to_owned(),
            manifest.display().to_string(),
            "--research-report-file".to_owned(),
            report_file.display().to_string(),
            "--retest-horizon-plan-output-file".to_owned(),
            plan_output.display().to_string(),
            "--retest-horizon-latest-l1-as-of-ms".to_owned(),
            "7201300".to_owned(),
            "--now-ms".to_owned(),
            "7400000".to_owned(),
        ]
        .into_iter(),
    )
    .expect("plan build args parse")
    .expect("plan build args returned");
    let summary = run(args).await.expect("plan builds");

    assert_eq!(summary.retest_horizon_plans_created, 1);
    assert_eq!(
        summary.output_files,
        vec![plan_output.display().to_string()]
    );
    let plan: Value =
        serde_json::from_slice(&fs::read(&plan_output).expect("plan")).expect("plan json");
    assert_eq!(
        plan["schema_version"],
        json!("research_retest_horizon_plan_v1")
    );
    assert_eq!(plan["generated_at_ms"], json!(7_400_000));
    assert_eq!(plan["latest_l1_as_of_ms"], json!(7_201_300));
    assert_eq!(plan["summary"]["candidate_count"], json!(1));
    assert_eq!(plan["summary"]["horizon_count"], json!(1));
    assert_eq!(
        plan["horizon_rows"][0]["next_action"],
        json!("accumulate_completed_native_replay_samples")
    );
}

#[tokio::test]
async fn retest_refresh_cycle_waits_without_writing_focused_manifest() {
    let root = test_root("retest-refresh-wait");
    let (manifest, report_file) = write_refresh_cycle_inputs(&root).await;
    let output = root.join("cycle-out");

    let args = parse_args(
        [
            "--run-retest-refresh-cycle".to_owned(),
            "--input-manifest-file".to_owned(),
            manifest.display().to_string(),
            "--research-report-file".to_owned(),
            report_file.display().to_string(),
            "--retest-horizon-latest-l1-as-of-ms".to_owned(),
            "1000".to_owned(),
            "--output-dir".to_owned(),
            output.display().to_string(),
            "--now-ms".to_owned(),
            "2000".to_owned(),
        ]
        .into_iter(),
    )
    .expect("refresh args parse")
    .expect("refresh args returned");
    let summary = run(args).await.expect("refresh cycle waits");

    assert_eq!(summary.retest_horizon_plans_created, 1);
    assert_eq!(summary.retest_horizon_statuses_validated, 1);
    assert_eq!(
        summary.retest_cycle_scheduler_action,
        Some("WAIT_UNTIL_MARKET_L1_HORIZON_MATERIALIZES".to_owned())
    );
    assert_eq!(summary.focused_retest_manifests_created, 0);
    assert!(output.join("retest-horizon-plan.json").exists());
    assert!(output.join("retest-horizon-status.json").exists());
    assert!(!output.join("research-input-manifest.json").exists());
}

#[tokio::test]
async fn retest_refresh_cycle_writes_focused_manifest_for_accumulation_ready_horizon() {
    let root = test_root("retest-refresh-run");
    let (manifest, report_file) = write_refresh_cycle_inputs(&root).await;
    let output = root.join("cycle-out");

    let args = parse_args(
        [
            "--run-retest-refresh-cycle".to_owned(),
            "--input-manifest-file".to_owned(),
            manifest.display().to_string(),
            "--research-report-file".to_owned(),
            report_file.display().to_string(),
            "--retest-horizon-latest-l1-as-of-ms".to_owned(),
            "7201300".to_owned(),
            "--output-dir".to_owned(),
            output.display().to_string(),
            "--research-packet-id".to_owned(),
            "refresh_cycle_focus".to_owned(),
            "--now-ms".to_owned(),
            "7400000".to_owned(),
        ]
        .into_iter(),
    )
    .expect("refresh args parse")
    .expect("refresh args returned");
    let summary = run(args).await.expect("refresh cycle writes focus");

    assert_eq!(summary.retest_horizon_plans_created, 1);
    assert_eq!(summary.retest_horizon_statuses_validated, 1);
    assert_eq!(
        summary.retest_cycle_scheduler_action,
        Some("RUN_FOCUSED_RETEST_RESEARCH".to_owned())
    );
    assert_eq!(summary.focused_retest_manifests_created, 1);
    assert_eq!(summary.focused_retest_candidate_bundle_refs, 1);
    assert!(output.join("retest-horizon-plan.json").exists());
    assert!(output.join("retest-horizon-status.json").exists());
    assert!(output.join("research-input-manifest.json").exists());
    assert!(output.join("research-input-manifest.summary.json").exists());
}

#[test]
fn focused_retest_dispatch_packet_id_is_stable_for_same_refresh_inputs() {
    let source_manifest: crate::model::ResearchInputManifest =
        serde_json::from_value(focused_retest_source_manifest_json())
            .expect("source manifest parses");
    let status = focused_retest_run_now_status_json();
    let mut args = default_args();
    args.input_manifest_s3_bucket = Some("research-bucket".to_owned());
    args.input_manifest_s3_key = Some(
        "research-input-manifest/schema=research_input_manifest_v1/source/manifest.json".to_owned(),
    );
    args.research_report_s3_bucket = Some("research-bucket".to_owned());
    args.research_report_s3_key =
        Some("research-run-report/schema=research_run_report_v1/report.json".to_owned());
    args.run_scope = "focused_retest_local_validation".to_owned();

    let build_a = crate::focused_retest::build_focused_retest_manifest(
        &status,
        &source_manifest,
        &crate::focused_retest::FocusedRetestBuildOptions {
            generated_at_ms: 7_400_000,
            research_packet_id: "research_focus_7400000".to_owned(),
            run_scope: "focused_retest_local_validation".to_owned(),
            next_actions: crate::focused_retest::default_focused_retest_actions(),
            candidate_lifecycle_key_filter: Vec::new(),
            historical_replay_index_ref_mode:
                crate::focused_retest::HistoricalReplayIndexRefMode::Auto,
            s3_write: true,
        },
    )
    .expect("focused build a succeeds");
    let build_b = crate::focused_retest::build_focused_retest_manifest(
        &status,
        &source_manifest,
        &crate::focused_retest::FocusedRetestBuildOptions {
            generated_at_ms: 7_500_000,
            research_packet_id: "research_focus_7500000".to_owned(),
            run_scope: "focused_retest_local_validation".to_owned(),
            next_actions: crate::focused_retest::default_focused_retest_actions(),
            candidate_lifecycle_key_filter: Vec::new(),
            historical_replay_index_ref_mode:
                crate::focused_retest::HistoricalReplayIndexRefMode::Auto,
            s3_write: true,
        },
    )
    .expect("focused build b succeeds");

    let first_id = focused_retest_dispatch_packet_id(&args, Some(7_201_300), &build_a)
        .expect("first dispatch id");
    let second_id = focused_retest_dispatch_packet_id(&args, Some(7_201_300), &build_b)
        .expect("second dispatch id");
    let advanced_l1_id = focused_retest_dispatch_packet_id(&args, Some(7_801_300), &build_b)
        .expect("advanced l1 dispatch id");

    assert_eq!(first_id, second_id);
    assert_ne!(first_id, advanced_l1_id);
    assert!(first_id.starts_with("research_focus_"));
    assert_eq!(
        focused_retest_dispatch_manifest_s3_key(&first_id)
            .expect("dispatch key")
            .as_str(),
        format!(
            "research-input-manifest/schema=research_input_manifest_v1/dedupe_key={first_id}/manifest.json"
        )
    );
}

#[test]
fn shadow_accumulation_dispatch_filters_manifest_to_deficient_lifecycle_keys() {
    let args = default_args();
    let state = retest_cycle_source_state();
    let source_manifest: crate::model::ResearchInputManifest =
        serde_json::from_value(focused_retest_source_manifest_json())
            .expect("source manifest parses");
    let status = focused_retest_run_now_status_json();

    let dispatch = build_shadow_accumulation_manifest_dispatch(
        &args,
        &state,
        &status,
        &source_manifest,
        Some(7_201_300),
        7_400_000,
        vec!["cand_focus:v1".to_owned(), "missing:v1".to_owned()],
    )
    .expect("shadow accumulation dispatch builds")
    .expect("shadow accumulation dispatch is selected");

    assert!(dispatch.key.starts_with(
        "research-input-manifest/schema=research_input_manifest_v1/dedupe_key=research_shadow_accumulation_"
    ));
    assert_eq!(
        dispatch.manifest.run_scope.as_deref(),
        Some("shadow_sample_accumulation_local_validation")
    );
    assert_eq!(dispatch.manifest.candidate_bundle_refs.len(), 1);
    assert!(
        dispatch.manifest.candidate_bundle_refs[0]
            .uri
            .contains("candidate_id=cand_focus")
    );
    assert_eq!(dispatch.manifest.historical_replay_run_index_refs.len(), 1);
    assert_eq!(dispatch.focused_horizon_count, 1);
    assert_eq!(dispatch.focused_candidate_bundle_refs, 1);
    assert_eq!(
        dispatch.deficit_lifecycle_keys,
        vec!["cand_focus:v1".to_owned(), "missing:v1".to_owned()]
    );
}

#[test]
fn shadow_accumulation_dispatch_skips_empty_deficit_keys() {
    let args = default_args();
    let state = retest_cycle_source_state();
    let source_manifest: crate::model::ResearchInputManifest =
        serde_json::from_value(focused_retest_source_manifest_json())
            .expect("source manifest parses");
    let status = focused_retest_run_now_status_json();

    let dispatch = build_shadow_accumulation_manifest_dispatch(
        &args,
        &state,
        &status,
        &source_manifest,
        Some(7_201_300),
        7_400_000,
        Vec::new(),
    )
    .expect("empty deficit keys are valid");

    assert!(dispatch.is_none());
}

fn retest_cycle_source_state() -> RetestCycleSourceState {
    RetestCycleSourceState {
        schema_version: RETEST_CYCLE_SOURCE_STATE_SCHEMA_VERSION.to_owned(),
        generated_at_ms: 7_300_000,
        research_packet_id: "source_packet".to_owned(),
        run_scope: "focused_retest_local_validation".to_owned(),
        source_manifest_s3_bucket: "research-bucket".to_owned(),
        source_manifest_s3_key:
            "research-input-manifest/schema=research_input_manifest_v1/source/manifest.json"
                .to_owned(),
        source_research_report_s3_bucket: "research-bucket".to_owned(),
        source_research_report_s3_key:
            "research-run-report/schema=research_run_report_v1/report.json".to_owned(),
        source_research_report_id: "research_report_source".to_owned(),
        source_candidate_ids: vec!["cand_focus".to_owned()],
        replay_run_id_count: 1,
        summary_findings_count: 1,
        shadow_validation_run_count: 0,
        paper_trade_candidate_count: 0,
        safety: RetestCycleSourceStateSafety {
            dispatcher_prefix: "research-input-manifest/".to_owned(),
            state_s3_write: true,
            ecs_task_started: false,
            shadow_paper_live_enabled: false,
        },
    }
}

async fn write_refresh_cycle_inputs(root: &Path) -> (PathBuf, PathBuf) {
    let bundle =
        root.join("candidate-evidence-bundle/priority=p0/candidate_id=cand_001/part-000001.json");
    let delta = root.join("delta.json");
    let regime = root.join("regime.json");
    let manifest = root.join("manifest.json");
    let research_output = root.join("research-out");

    write_json(&bundle, &bundle_json_with_gate_inputs(1, 1_300));
    write_json(
        &delta,
        &json!([market_delta_json("delta_001", 1_300, 3_601_300, 0.021)]),
    );
    write_json(
        &regime,
        &json!([market_regime_json("regime_001", 1_300, 3_601_300)]),
    );
    write_json(
        &manifest,
        &json!({
            "schema_version": "research_input_manifest_v1",
            "research_packet_id": "manifest_packet",
            "run_scope": "manifest_batch",
            "candidate_bundle_refs": [{ "uri": bundle.display().to_string() }],
            "market_feature_delta_refs": [{ "uri": delta.display().to_string() }],
            "market_regime_context_refs": [{ "uri": regime.display().to_string() }],
            "runtime_budget_policy": {
                "max_candidate_bundle_count": 10,
                "max_market_artifact_ref_count": 10,
                "max_historical_replay_run_ref_count": 10,
                "max_replay_run_count": 20
            }
        }),
    );
    let research_summary = run(Args {
        input_manifest_file: Some(manifest.clone()),
        output_dir: Some(research_output),
        now_ms: Some(7_300_000),
        ..default_args()
    })
    .await
    .expect("research report builds");
    let report_file = output_file_containing(&research_summary, "research-run-report");
    (manifest, report_file)
}
