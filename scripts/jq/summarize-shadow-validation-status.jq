include "summarize-shadow-validation-status-common";
include "summarize-shadow-validation-status-sections";

records as $runs
| ($horizon_status_input[0] // null) as $horizon_status
| ($runs | map(select(status_value == "pending"))) as $pending
| ($runs | map(select(status_value == "completed"))) as $completed
| ($runs | map(select(status_value == "failed"))) as $failed
| ($runs | map(select(is_completed_passed_shadow))) as $paper_eligible
| ($runs | map(select((.termination_policy.no_order_execution // false) != true))) as $order_execution_violations
| ($runs | map(select((.paper_trade_candidate_contract_version // "") != "paper_trade_candidate_v1"))) as $paper_contract_mismatches
| {
    schema_version:"research_shadow_validation_status_checkpoint_v1",
    generated_at:$generated_at,
    shadow_validation_run_file:$shadow_validation_run_file,
    retest_horizon_status_file:(if $horizon_status_file == "" then null else $horizon_status_file end),
    safety:safety_section,
    upstream_state:upstream_state_section($horizon_status),
    stage_state:stage_state_section(
      $runs;
      $completed;
      $paper_eligible;
      $order_execution_violations;
      $paper_contract_mismatches;
      $horizon_status
    ),
    shadow_validation_summary:shadow_validation_summary_section(
      $runs;
      $pending;
      $completed;
      $failed;
      $paper_eligible;
      $paper_contract_mismatches;
      $order_execution_violations
    ),
    paper_gate:paper_gate_section(
      $pending;
      $failed;
      $paper_eligible;
      $paper_contract_mismatches;
      $order_execution_violations
    ),
    by_symbol:by_symbol_section($runs),
    by_candidate_lifecycle_key:by_candidate_lifecycle_key_section($runs)
  }
