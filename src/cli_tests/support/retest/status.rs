use serde_json::{Value, json};

pub(in crate::cli::tests) fn retest_horizon_wait_status_json() -> Value {
    json!({
        "schema_version": "research_horizon_status_checkpoint_v1",
        "generated_at": "2026-05-25T12:09:40Z",
        "safety": {
            "s3_write": false,
            "ecs_task_started": false,
            "dispatcher_mode_changed": false,
            "local_summary_only": true,
            "shadow_paper_live_enabled": false
        },
        "stage_state": {
            "candidate_generated": true,
            "research_replay_completed": true,
            "promotion_passed": false,
            "shadow_created": false,
            "paper_created": false,
            "live_enabled": false
        },
        "next_decision": {
            "verdict": "WAIT_FOR_MARKET_L1_HORIZON",
            "scheduler_hint": {
                "latest_l1_as_of_ms": 1_779_710_400_000_i64,
                "latest_l1_as_of_iso": "2026-05-25T12:00:00Z",
                "run_research_after_l1_as_of_ms": 1_779_719_361_452_i64,
                "run_research_after_l1_as_of_iso": "2026-05-25T14:29:21Z",
                "wait_deficit_ms": 8_961_452,
                "run_now_replay_ready": false,
                "promotion_ready_for_review": false
            },
            "blocked_actions": [
                "do_not_create_shadow_without_promotion",
                "do_not_create_paper_without_passed_shadow",
                "do_not_enable_live_from_research_batch"
            ]
        }
    })
}

pub(in crate::cli::tests) fn focused_retest_status_json() -> Value {
    let mut status = retest_horizon_wait_status_json();
    status["by_symbol"] = json!([
        {
            "symbol": "AAVE",
            "candidates": [
                {
                    "candidate_id": "cand_focus",
                    "candidate_lifecycle_key": "cand_focus:v1",
                    "hypothesis_type": "event_reaction",
                    "research_priority": "p0",
                    "horizons": [
                        {
                            "horizon": "1h",
                            "next_action": "accumulate_completed_native_replay_samples",
                            "symbols": ["AAVE"],
                            "replay_run_count": 2,
                            "completed_count": 1,
                            "completed_sample_deficit": 2,
                            "reason_codes": ["sample_deficit"]
                        }
                    ]
                },
                {
                    "candidate_id": "cand_wait",
                    "candidate_lifecycle_key": "cand_wait:v1",
                    "hypothesis_type": "event_reaction",
                    "research_priority": "p0",
                    "horizons": [
                        {
                            "horizon": "4h",
                            "next_action": "wait_for_market_l1_horizon",
                            "symbols": ["AAVE"],
                            "reason_codes": ["waiting_for_l1"]
                        }
                    ]
                }
            ]
        }
    ]);
    status
}

pub(in crate::cli::tests) fn focused_retest_run_now_status_json() -> Value {
    let mut status = focused_retest_status_json();
    status["next_decision"]["verdict"] = json!("REPLAY_READY_FOR_SOME_HORIZONS");
    status["next_decision"]["scheduler_hint"]["run_now_replay_ready"] = json!(true);
    status["next_decision"]["scheduler_hint"]["run_research_after_l1_as_of_ms"] = Value::Null;
    status["next_decision"]["scheduler_hint"]["run_research_after_l1_as_of_iso"] = Value::Null;
    status["next_decision"]["scheduler_hint"]["wait_deficit_ms"] = Value::Null;
    status
}
