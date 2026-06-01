use serde_json::{Value, json};

pub(in crate::cli::tests) fn focused_retest_source_manifest_json() -> Value {
    json!({
        "schema_version": "research_input_manifest_v1",
        "research_packet_id": "source_packet",
        "run_scope": "current_approved",
        "candidate_bundle_refs": [
            {
                "uri": "s3://bucket/candidate-evidence-bundle/priority=p0/candidate_id=cand_focus/part-000001.jsonl"
            },
            {
                "uri": "s3://bucket/candidate-evidence-bundle/priority=p0/candidate_id=cand_wait/part-000001.jsonl"
            }
        ],
        "historical_replay_run_index_refs": [
            {
                "uri": "s3://research/replay-run-index/part-000001.jsonl"
            }
        ],
        "runtime_budget_policy": {
            "max_candidate_bundle_count": 10,
            "max_market_artifact_ref_count": 20,
            "max_shadow_validation_run_ref_count": 20,
            "max_hypothesis_harness_result_ref_count": 20,
            "max_oss_adapter_run_ref_count": 20,
            "max_historical_replay_run_ref_count": 20,
            "max_replay_run_count": 100
        }
    })
}
