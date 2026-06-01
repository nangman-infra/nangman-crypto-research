use super::super::*;

#[tokio::test]
async fn market_s3_key_budget_is_enforced_before_reading_s3_objects() {
    let mut bundle = bundle_json();
    bundle["selected_market_artifacts"] = json!([
        {
            "artifact_type": "market_feature_delta",
            "artifact_id": "delta_001",
            "artifact_key": "market_feature_delta/run_id=l1_selected/delta.json",
            "l1_run_id": "l1_selected",
            "symbol_canonical": "SUI",
            "metric_name": "price",
            "window_start_ms": 1_000,
            "window_end_ms": 1_300,
            "known_as_of_ms": 1_300,
            "quality_status": "available"
        },
        {
            "artifact_type": "market_regime_context",
            "artifact_id": "regime_001",
            "artifact_key": "market_regime_context/run_id=l1_selected/context.json",
            "l1_run_id": "l1_selected",
            "scope": "market",
            "window_start_ms": 1_000,
            "window_end_ms": 1_300,
            "known_as_of_ms": 1_300,
            "quality_status": "available"
        }
    ]);
    let bundles =
        vec![serde_json::from_value(bundle).expect("candidate bundle test json matches model")];
    let args = Args {
        input_bundle_s3_bucket: Some(
            "nangman-crypto-dev-intel-candidate-<account-suffix>".to_owned(),
        ),
        input_bundle_s3_key: Some(
            "candidate-evidence-bundle/priority=p0/part-000001.jsonl".to_owned(),
        ),
        research_packet_id: "packet_test".to_owned(),
        run_scope: "test".to_owned(),
        now_ms: Some(0),
        ..default_args()
    };

    let delta_error = load_market_deltas(&args, &bundles, None, 0)
        .await
        .expect_err("market delta key budget fails before S3 read");
    assert!(
        delta_error
            .to_string()
            .contains("market_feature_delta_s3_key_count")
    );

    let context_error = load_regime_contexts(&args, &bundles, None, 0)
        .await
        .expect_err("market context key budget fails before S3 read");
    assert!(
        context_error
            .to_string()
            .contains("market_regime_context_s3_key_count")
    );
}
