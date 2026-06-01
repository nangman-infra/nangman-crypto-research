use super::*;

pub(in crate::cli::tests) fn bundle_json() -> Value {
    json!({
        "candidate_id": "cand_001",
        "candidate_lifecycle_key": "cand_001:v1",
        "bundle_key": "candidate-evidence-bundle/priority=p0/schema=intel_candidate_evidence_bundle_v1/part-000001.jsonl",
        "producer_app": "intel-candidate-app",
        "producer_run_id": "run_001",
        "created_at_ms": 7_200_000,
        "event_time_ms": 1_000,
        "published_at_ms": 1_000,
        "fetched_at_ms": 1_100,
        "structured_at_ms": 1_200,
        "candidate_created_at_ms": 1_300,
        "decision_available_at_ms": 1_300,
        "forbidden_lookahead_boundary_ms": 1_300,
        "schema_version": "intel_candidate_evidence_bundle_v1",
        "scoring_policy_version": "scoring-policy.v1",
        "normalized_symbols": ["SUI"],
        "symbol_universe_snapshot_id": "universe_001",
        "universe_as_of_ms": 1_200,
        "approved_universe_symbol": true,
        "event_types": ["project_notice"],
        "hypothesis_type": "event_reaction",
        "allowed_horizons": ["1h"],
        "source_story_cluster_ids": ["cluster_001"],
        "source_structured_packet_ids": ["packet_001"],
        "source_context_flag_packet_ids": [],
        "evidence_refs": ["packet_001"],
        "metric_evidence": [],
        "data_quality_summary": {
            "market_data_quality_summary_key": "market_data_quality_summary/run/summary.json",
            "status": "available"
        },
        "selected_market_artifacts": [],
        "candidate_class": "research_candidate",
        "candidate_score": 72,
        "research_priority": "p0",
        "research_eligible": true,
        "validation_requirements": {
            "required_adapters": ["native_replay"],
            "optional_adapters": ["freqtrade_style"],
            "min_unseen_windows": 1,
            "include_fee": true,
            "include_slippage": true,
            "include_latency_assumption": true,
            "include_liquidity_filter": true,
            "required_train_validation_split": true,
            "max_adapter_runtime_minutes": 15
        },
        "source_independence": {
            "source_event_count": 1,
            "independent_source_count": 1,
            "official_source_present": true,
            "duplicate_content_hashes": [],
            "original_source_ids": ["official"]
        },
        "symbol_resolution_trace": [{
            "raw_mentions": ["SUI"],
            "resolved_project": "Sui",
            "resolved_asset": "SUI",
            "canonical_symbol": "SUI",
            "venue_symbols": ["SUIUSDT"],
            "mapping_confidence": "strong"
        }],
        "confidence_summary": {},
        "observe_or_reject_reasons": [],
        "parent_artifact_ids": ["packet_001"],
        "storage_uri": "s3://bucket/key",
        "checksum": "checksum",
        "idempotency_key": "idem_001"
    })
}

pub(in crate::cli::tests) fn bundle_json_with_gate_inputs(index: usize, decision_ms: i64) -> Value {
    let mut bundle = bundle_json();
    bundle["candidate_id"] = json!(format!("cand_{index:03}"));
    bundle["candidate_lifecycle_key"] = json!(format!("cand_{index:03}:v1"));
    bundle["idempotency_key"] = json!(format!("idem_{index:03}"));
    bundle["created_at_ms"] = json!(decision_ms);
    bundle["event_time_ms"] = json!(decision_ms - 300);
    bundle["published_at_ms"] = json!(decision_ms - 250);
    bundle["fetched_at_ms"] = json!(decision_ms - 200);
    bundle["structured_at_ms"] = json!(decision_ms - 100);
    bundle["candidate_created_at_ms"] = json!(decision_ms);
    bundle["decision_available_at_ms"] = json!(decision_ms);
    bundle["forbidden_lookahead_boundary_ms"] = json!(decision_ms);
    bundle["universe_as_of_ms"] = json!(decision_ms - 100);
    bundle["validation_requirements"]["min_unseen_windows"] = json!(1);
    bundle["validation_requirements"]["include_liquidity_filter"] = json!(false);
    bundle["validation_requirements"]["required_train_validation_split"] = json!(true);
    bundle
}

pub(in crate::cli::tests) fn retarget_bundle_symbol(bundle: &mut Value, symbol: &str) {
    bundle["normalized_symbols"] = json!([symbol]);
    bundle["symbol_resolution_trace"][0]["raw_mentions"] = json!([symbol]);
    bundle["symbol_resolution_trace"][0]["resolved_project"] = json!(symbol);
    bundle["symbol_resolution_trace"][0]["resolved_asset"] = json!(symbol);
    bundle["symbol_resolution_trace"][0]["canonical_symbol"] = json!(symbol);
    bundle["symbol_resolution_trace"][0]["venue_symbols"] = json!([format!("{symbol}USDT")]);
}
