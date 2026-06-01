{
  schema_version,
  research_run_report_id,
  source_candidate_count:(.source_candidate_ids | length),
  replay_run_count:(.replay_run_ids | length),
  partition_count,
  top_symbols,
  surviving_candidate_count:(.surviving_candidate_keys | length),
  retest_candidate_count:(.retest_candidate_keys | length),
  pruned_candidate_count:(.pruned_candidate_keys | length),
  shadow_validation_count:(.shadow_validation_runs | length),
  paper_trade_candidate_count:(.paper_trade_candidates | length)
}
