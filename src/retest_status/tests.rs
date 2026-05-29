use super::*;
use crate::model::RETEST_HORIZON_STATUS_SCHEMA_VERSION;
use serde_json::{Value, json};

fn plan() -> Value {
    json!({
        "schema_version": "research_retest_horizon_plan_v1",
        "latest_l1_as_of_ms": 1_779_710_400_000_i64,
        "horizon_rows": [
            {
                "candidate_id": "cand_a",
                "candidate_lifecycle_key": "cand_a:v1",
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
                "train_validation_split_materialized": true,
                "liquidity_filter_materialized_count": 1,
                "missing_market_replay_data_count": 0,
                "gate_biases": ["RETEST_BIAS"],
                "reason_codes": ["sample_deficit"],
                "next_action": "accumulate_completed_native_replay_samples"
            },
            {
                "candidate_id": "cand_a",
                "candidate_lifecycle_key": "cand_a:v1",
                "symbols": ["AAVE"],
                "primary_symbol": "AAVE",
                "hypothesis_type": "event_reaction",
                "research_priority": "p0",
                "horizon": "4h",
                "horizon_due_ms": 1_779_719_361_452_i64,
                "horizon_market_data_materialized": false,
                "replay_run_count": 2,
                "completed_count": 0,
                "completed_sample_deficit": 30,
                "inferred_unseen_window_count": 1,
                "unseen_window_deficit": 19,
                "reason_codes": ["waiting_for_l1"],
                "next_action": "wait_for_market_l1_horizon"
            }
        ]
    })
}

#[test]
fn builds_run_status_when_some_horizons_can_accumulate_samples() {
    let status = build_retest_horizon_status(
        &plan(),
        None,
        &RetestHorizonStatusBuildOptions {
            generated_at_ms: 1_779_714_000_000,
            plan_file: Some("/tmp/plan.json".to_owned()),
            driver_summary_file: None,
            checkpoint_s3_write: false,
        },
    )
    .expect("status builds");

    assert_eq!(
        status["schema_version"],
        json!(RETEST_HORIZON_STATUS_SCHEMA_VERSION)
    );
    assert_eq!(status["verdict"], json!("REPLAY_READY_FOR_SOME_HORIZONS"));
    assert_eq!(
        status["next_decision"]["scheduler_hint"]["run_now_replay_ready"],
        json!(true)
    );
    assert_eq!(
        status["next_decision"]["scheduler_hint"]["run_research_after_l1_as_of_ms"],
        json!(1_779_719_361_452_i64)
    );
    assert_eq!(status["by_symbol"][0]["symbol"], json!("AAVE"));
    assert_eq!(
        status["by_symbol"][0]["candidates"][0]["horizons"][1]["next_action"],
        json!("wait_for_market_l1_horizon")
    );
}

#[test]
fn rejects_plan_without_rows() {
    let error = build_retest_horizon_status(
        &json!({"schema_version": "research_retest_horizon_plan_v1"}),
        None,
        &RetestHorizonStatusBuildOptions {
            generated_at_ms: 1,
            plan_file: None,
            driver_summary_file: None,
            checkpoint_s3_write: false,
        },
    )
    .expect_err("rows are required");
    assert!(error.to_string().contains("horizon_rows"));
}
