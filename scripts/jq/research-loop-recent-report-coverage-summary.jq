{
  schema_version:"research_recent_report_coverage_summary_v1",
  selection:"recent_research_reports",
  report_read_count:length,
  replayed_symbols:(map((.partition_symbols // [])[]?, (.top_symbols // [])[]?) | unique | sort),
  replayed_symbol_count:(map((.partition_symbols // [])[]?, (.top_symbols // [])[]?) | unique | length),
  latest_last_modified:(map(.last_modified) | max // null),
  statuses:(map(.research_run_status) | unique | sort),
  run_scopes:(map(.run_scope // "unknown") | unique | sort)
}
