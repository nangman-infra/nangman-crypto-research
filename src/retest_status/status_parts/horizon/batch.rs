use serde_json::{Value, json};

pub(in crate::retest_status) fn batch_state(driver: &Value) -> Value {
    json!({
        "run_id": driver.get("run_id").cloned().unwrap_or(Value::Null),
        "universe_mode": driver.pointer("/manifest/universe_mode").cloned().unwrap_or(Value::Null),
        "dispatch_mode": driver.pointer("/manifest/dispatch_mode").cloned().unwrap_or(Value::Null),
        "selected_candidate_count": driver.pointer("/manifest/selected_candidate_count").cloned().unwrap_or(Value::Null),
        "eligible_candidate_pool_count": driver.pointer("/manifest/eligible_candidate_pool_count").cloned().unwrap_or(Value::Null),
        "selected_candidate_limit_reached": driver.pointer("/manifest/selected_candidate_limit_reached").cloned().unwrap_or(Value::Null),
        "unselected_eligible_candidate_count": driver.pointer("/manifest/unselected_eligible_candidate_count").cloned().unwrap_or(Value::Null),
        "selected_current_approved_candidate_count": driver.pointer("/manifest/selected_current_approved_candidate_count").cloned().unwrap_or(Value::Null),
        "research_report_status": driver.pointer("/report/research_run_status").cloned().unwrap_or(Value::Null),
        "source_candidate_count": driver.pointer("/report/source_candidate_count").cloned().unwrap_or(Value::Null),
        "replay_run_count": driver.pointer("/report/replay_run_count").cloned().unwrap_or(Value::Null),
        "retest_candidate_count": driver.pointer("/report/retest_candidate_count").cloned().unwrap_or(Value::Null),
        "surviving_candidate_count": driver.pointer("/report/surviving_candidate_count").cloned().unwrap_or(Value::Null),
        "shadow_validation_count": driver.pointer("/report/shadow_validation_count").cloned().unwrap_or(Value::Null),
        "paper_trade_candidate_count": driver.pointer("/report/paper_trade_candidate_count").cloned().unwrap_or(Value::Null)
    })
}
