.schema_version == "research_run_report_v1"
and (.research_run_report_id | type == "string" and length > 0)
and (.source_candidate_ids | type == "array" and length > 0)
and (.replay_run_ids | type == "array" and length > 0)
and (.partition_aggregates | type == "array")
and (.research_gate_policy.policy_version | type == "string" and length > 0)
