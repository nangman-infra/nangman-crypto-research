use serde_json::{Value, json};

pub(in crate::cli::tests) fn retest_horizon_plan_json() -> Value {
    json!({
        "schema_version": "research_retest_horizon_plan_v1",
        "latest_l1_as_of_ms": 1_779_710_400_000_i64,
        "horizon_rows": [
            {
                "candidate_id": "cand_focus",
                "candidate_lifecycle_key": "cand_focus:v1",
                "symbols": ["AAVE"],
                "primary_symbol": "AAVE",
                "hypothesis_type": "event_reaction",
                "research_priority": "p0",
                "horizon": "1h",
                "horizon_due_ms": 1_779_710_300_000_i64,
                "horizon_market_data_materialized": true,
                "replay_run_count": 2,
                "completed_count": 1,
                "completed_sample_deficit": 29,
                "inferred_unseen_window_count": 1,
                "unseen_window_deficit": 19,
                "reason_codes": ["sample_deficit"],
                "next_action": "accumulate_completed_native_replay_samples"
            },
            {
                "candidate_id": "cand_wait",
                "candidate_lifecycle_key": "cand_wait:v1",
                "symbols": ["AAVE"],
                "primary_symbol": "AAVE",
                "hypothesis_type": "event_reaction",
                "research_priority": "p0",
                "horizon": "4h",
                "horizon_due_ms": 1_779_719_361_452_i64,
                "horizon_market_data_materialized": false,
                "replay_run_count": 0,
                "completed_count": 0,
                "completed_sample_deficit": 30,
                "inferred_unseen_window_count": 0,
                "unseen_window_deficit": 20,
                "reason_codes": ["waiting_for_l1"],
                "next_action": "wait_for_market_l1_horizon"
            }
        ]
    })
}
