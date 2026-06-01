include "retest-horizon-plan-rows";
include "retest-horizon-plan-summary";

($bundles_file[0] // []) as $bundles
| . as $report
| ($report.research_gate_policy.min_completed_samples_for_shadow // 30) as $min_completed
| ($report.partition_aggregates // []) as $aggregates
| latest_as_of as $latest_l1
| retest_horizon_rows($bundles; $report; $min_completed; $aggregates; $latest_l1) as $horizon_rows
| {
    schema_version:"research_retest_horizon_plan_v1",
    manifest_file:$manifest_file,
    report_file:$report_file,
    latest_l1_as_of_ms:$latest_l1,
    research_gate_policy:$report.research_gate_policy,
    summary:retest_horizon_plan_summary($bundles; $horizon_rows),
    by_candidate:retest_horizon_by_candidate($horizon_rows),
    horizon_rows:$horizon_rows
  }
