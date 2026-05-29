use super::*;
use serde_json::json;

#[test]
fn validates_wait_status_handoff() {
    let status = json!({
        "schema_version": "research_horizon_status_checkpoint_v1",
        "safety": {
            "s3_write": false,
            "ecs_task_started": false,
            "dispatcher_mode_changed": false,
            "local_summary_only": true,
            "shadow_paper_live_enabled": false
        },
        "stage_state": {
            "promotion_passed": false,
            "paper_created": false,
            "live_enabled": false
        },
        "next_decision": {
            "verdict": "WAIT_FOR_MARKET_L1_HORIZON",
            "scheduler_hint": {
                "latest_l1_as_of_ms": 1779710400000_i64,
                "run_research_after_l1_as_of_ms": 1779719361452_i64,
                "run_now_replay_ready": false,
                "promotion_ready_for_review": false
            },
            "blocked_actions": [
                "do_not_create_shadow_without_promotion",
                "do_not_create_paper_without_passed_shadow",
                "do_not_enable_live_from_research_batch"
            ]
        }
    });

    let summary = validate_retest_horizon_status(&status).expect("status validates");
    assert_eq!(
        summary.scheduler_action,
        "WAIT_UNTIL_MARKET_L1_HORIZON_MATERIALIZES"
    );
    assert_eq!(summary.run_not_before_ms, Some(1779719361452));
}

#[test]
fn rejects_wait_status_without_not_before_time() {
    let status = json!({
        "schema_version": "research_horizon_status_checkpoint_v1",
        "safety": {
            "s3_write": false,
            "ecs_task_started": false,
            "dispatcher_mode_changed": false,
            "local_summary_only": true,
            "shadow_paper_live_enabled": false
        },
        "stage_state": {
            "live_enabled": false
        },
        "next_decision": {
            "verdict": "WAIT_FOR_MARKET_L1_HORIZON",
            "scheduler_hint": {
                "run_now_replay_ready": false,
                "promotion_ready_for_review": false
            },
            "blocked_actions": [
                "do_not_create_shadow_without_promotion",
                "do_not_create_paper_without_passed_shadow",
                "do_not_enable_live_from_research_batch"
            ]
        }
    });

    let error = validate_retest_horizon_status(&status).expect_err("wait time is required");
    assert!(error.to_string().contains("run_research_after_l1_as_of_ms"));
}

#[test]
fn rejects_status_that_enables_live() {
    let status = json!({
        "schema_version": "research_horizon_status_checkpoint_v1",
        "safety": {
            "s3_write": false,
            "ecs_task_started": false,
            "dispatcher_mode_changed": false,
            "local_summary_only": true,
            "shadow_paper_live_enabled": false
        },
        "stage_state": {
            "live_enabled": true
        },
        "next_decision": {
            "verdict": "INSPECT_REMAINING_GATE_REASONS",
            "scheduler_hint": {},
            "blocked_actions": [
                "do_not_create_shadow_without_promotion",
                "do_not_create_paper_without_passed_shadow",
                "do_not_enable_live_from_research_batch"
            ]
        }
    });

    let error = validate_retest_horizon_status(&status).expect_err("live is rejected");
    assert!(error.to_string().contains("live trading"));
}
