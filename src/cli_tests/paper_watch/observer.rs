use super::*;

#[test]
fn paper_watch_observer_parses_service_mode_with_durable_nats_consumer() {
    let args = parse_args(
        [
            "--run-paper-watch-observer",
            "--paper-watch-candidate-s3-bucket",
            "research-bucket",
            "--paper-watch-candidate-s3-prefix",
            "paper-watch-candidate/schema=paper_watch_candidate_v1",
            "--market-live-nats-url",
            "nats://127.0.0.1:4222",
            "--output-s3-bucket",
            "research-bucket",
            "--output-s3-prefix",
            "paper-watch-observer-state/schema=paper_watch_observer_snapshot_v1",
            "--paper-watch-observer-max-iterations",
            "0",
        ]
        .into_iter()
        .map(str::to_owned),
    )
    .expect("observer mode parses")
    .expect("observer args are returned");

    let config = paper_watch_observer_nats_config(&args).expect("observer nats config is valid");
    assert!(args.run_paper_watch_observer);
    assert_eq!(args.paper_watch_observer_max_iterations, 0);
    assert_eq!(config.consumer, "research-paper-watch-observer");
    assert_eq!(config.deliver_policy, "new");
    assert!(!config.delete_consumer_after_read);
}

#[test]
fn paper_watch_observer_rejects_one_shot_candidate_key() {
    let err = parse_args(
        [
            "--run-paper-watch-observer",
            "--paper-watch-candidate-s3-bucket",
            "research-bucket",
            "--paper-watch-candidate-s3-key",
            "paper-watch-candidate/schema=paper_watch_candidate_v1/part-000001.jsonl",
            "--market-live-nats-url",
            "nats://127.0.0.1:4222",
            "--output-s3-bucket",
            "research-bucket",
        ]
        .into_iter()
        .map(str::to_owned),
    )
    .expect_err("observer should reject one-shot candidate keys");

    assert!(
        err.to_string()
            .contains("--run-paper-watch-observer uses --paper-watch-candidate-s3-prefix")
    );
}

#[tokio::test]
async fn paper_watch_observer_writes_local_marks_and_snapshot() {
    let root = test_root("paper-watch-observer-local-output");
    let args = Args {
        output_dir: Some(root.clone()),
        output_s3_bucket: None,
        output_s3_prefix: Some(
            "paper-watch-observer-state/schema=paper_watch_observer_snapshot_v1".to_owned(),
        ),
        ..default_args()
    };
    let candidates = serde_json::from_value::<Vec<crate::model::PaperWatchCandidate>>(json!([
        paper_watch_candidate_json("watch_xrp", "XRP")
    ]))
    .expect("paper watch candidates parse");
    let ticks = serde_json::from_value::<Vec<crate::model::MarketLiveTick>>(json!([
        market_live_tick_json("tick_001", "XRP", 2_000, 1.0),
        market_live_tick_json("tick_002", "XRP", 3_000, 1.01)
    ]))
    .expect("market live ticks parse");
    let marks = crate::paper_live::build_paper_watch_live_marks(&candidates, &ticks);

    let mark_files = write_paper_watch_observer_live_marks(&args, &marks, 3_600_000)
        .await
        .expect("observer local marks are written");
    let snapshot_file = write_paper_watch_observer_snapshot(
        &args,
        &json!({
            "schema_version": "paper_watch_observer_snapshot_v1",
            "active_candidate_count": 1,
            "safety": {
                "paper_only": true,
                "live_enabled": false,
                "order_execution_enabled": false,
                "execution_approval_emitted": false
            }
        }),
        3_600_000,
    )
    .await
    .expect("observer local snapshot is written");

    assert_eq!(mark_files.len(), 1);
    let mark_bytes = fs::read(&mark_files[0]).expect("mark file exists");
    let parsed_marks =
        crate::io::read_paper_watch_live_marks_from_bytes("observer-test", &mark_bytes)
            .expect("paper watch live marks parse");
    assert_eq!(parsed_marks.len(), 2);
    assert!(snapshot_file.contains("paper-watch-observer-state/"));
    let snapshot: Value =
        serde_json::from_slice(&fs::read(snapshot_file).expect("snapshot file exists"))
            .expect("snapshot json parses");
    assert_eq!(snapshot["safety"]["order_execution_enabled"], json!(false));
}
