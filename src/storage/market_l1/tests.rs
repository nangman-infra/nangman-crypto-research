use super::contract::{
    is_success_l1_index_pointer, is_success_l1_manifest, l1_artifact_key_from_manifest,
    l1_index_pointer_key, l1_manifest_key_from_pointer,
};
use serde_json::json;

#[test]
fn l1_index_pointer_key_matches_market_ingest_partition_contract() {
    assert_eq!(
        l1_index_pointer_key(1_778_387_400_000).expect("valid timestamp"),
        "l1_index/window_ms=1000/event_date=2026-05-10/hour=04/window_start_ms=1778387400000.json"
    );
}

#[test]
fn extracts_manifest_and_artifact_keys_from_l1_index_contract() {
    let pointer = json!({
        "schema_version": "l1_index_pointer_v1",
        "canonical_manifest_key": "s3://bucket/runs/run_id=l1_1_2_3/manifest.json",
        "status": "success"
    });
    let manifest = json!({
        "schema_version": "l1_manifest_v1",
        "status": "success",
        "market_feature_delta_key": "s3://bucket/market_feature_delta/run_id=l1_1_2_3/delta.json",
        "market_regime_context_key": "market_regime_context/run_id=l1_1_2_3/context.json"
    });

    assert!(is_success_l1_index_pointer(&pointer));
    assert_eq!(
        l1_manifest_key_from_pointer(&pointer),
        Some("runs/run_id=l1_1_2_3/manifest.json".to_owned())
    );
    assert!(is_success_l1_manifest(&manifest));
    assert_eq!(
        l1_artifact_key_from_manifest(&manifest, "market_feature_delta_key"),
        Some("market_feature_delta/run_id=l1_1_2_3/delta.json".to_owned())
    );
    assert_eq!(
        l1_artifact_key_from_manifest(&manifest, "market_regime_context_key"),
        Some("market_regime_context/run_id=l1_1_2_3/context.json".to_owned())
    );
}

#[test]
fn ignores_non_success_l1_index_or_manifest() {
    let pointer = json!({
        "schema_version": "l1_index_pointer_v1",
        "canonical_manifest_key": "runs/run_id=l1_1_2_3/manifest.json",
        "status": "failed"
    });
    let manifest = json!({
        "schema_version": "l1_manifest_v1",
        "status": "failed",
        "market_feature_delta_key": "market_feature_delta/run_id=l1_1_2_3/delta.json"
    });

    assert!(!is_success_l1_index_pointer(&pointer));
    assert!(!is_success_l1_manifest(&manifest));
}
