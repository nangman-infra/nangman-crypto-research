use super::super::*;

#[tokio::test]
async fn derives_market_l1_s3_keys_from_candidate_bundle() {
    let mut bundle = bundle_json();
    bundle["data_quality_summary"]["market_data_quality_summary_key"] =
        json!("market_data_quality_summary/run_id=l1_001/summary.json");
    bundle["selected_market_artifacts"] = json!([
        {
            "artifact_type": "market_feature_delta",
            "artifact_id": "delta_002",
            "artifact_key": "s3://nangman-crypto-dev-market-ingest-l1-<account-suffix>/market_feature_delta/run_id=l1_direct/delta.json",
            "l1_run_id": "l1_direct",
            "symbol_canonical": "SUI",
            "metric_name": "price",
            "window_start_ms": 1_000,
            "window_end_ms": 1_300,
            "known_as_of_ms": 1_300,
            "quality_status": "available"
        },
        {
            "artifact_type": "market_feature_delta_summary",
            "artifact_id": "delta_summary_001",
            "artifact_key": "market_feature_delta_summary/run_id=l1_selected/summary.json",
            "l1_run_id": "l1_selected",
            "symbol_canonical": "SUI",
            "metric_name": "price",
            "window_start_ms": 1_000,
            "window_end_ms": 1_300,
            "known_as_of_ms": 1_300,
            "quality_status": "available"
        },
        {
            "artifact_type": "market_feature_delta_summary",
            "artifact_id": "delta_summary_002",
            "artifact_key": "market_feature_delta_summary/run_id=l1_summary_key_only/summary.json",
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
            "artifact_key": "s3://nangman-crypto-dev-market-ingest-l1-<account-suffix>/market_regime_context/run_id=l1_selected/context.json",
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
        market_feature_delta_s3_keys: vec![
            "market_feature_delta/run_id=l1_cli/delta.json".to_owned(),
        ],
        market_regime_context_s3_keys: vec![
            "market_regime_context/run_id=l1_cli/context.json".to_owned(),
        ],
        research_packet_id: "packet_test".to_owned(),
        run_scope: "test".to_owned(),
        now_ms: Some(0),
        ..default_args()
    };

    assert_eq!(
        market_feature_delta_s3_keys(&args, &bundles)
            .await
            .expect("market feature delta keys derive"),
        vec![
            "market_feature_delta/run_id=l1_001/delta.json",
            "market_feature_delta/run_id=l1_cli/delta.json",
            "market_feature_delta/run_id=l1_direct/delta.json",
            "market_feature_delta/run_id=l1_selected/delta.json",
            "market_feature_delta/run_id=l1_summary_key_only/delta.json",
        ]
    );
    assert_eq!(
        market_regime_context_s3_keys(&args, &bundles)
            .await
            .expect("market regime context keys derive"),
        vec![
            "market_regime_context/run_id=l1_001/context.json",
            "market_regime_context/run_id=l1_cli/context.json",
            "market_regime_context/run_id=l1_selected/context.json",
        ]
    );
}
