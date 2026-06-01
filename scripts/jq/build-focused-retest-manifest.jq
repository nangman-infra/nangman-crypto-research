include "build-focused-retest-manifest-common";
include "build-focused-retest-manifest-selection";
include "build-focused-retest-manifest-output";

($status[0]) as $status_doc
| ($source[0]) as $source_manifest
| (action_list($focus_next_actions)) as $actions
| (focused_retest_rows($status_doc; $actions)) as $focus_rows
| ($focus_rows | map(.candidate_id) | unique_sorted) as $focus_candidate_ids
| (carry_historical_index_refs($include_historical_index_refs; $actions)) as $carry_historical_index_refs
| (source_candidate_refs($source_manifest)) as $source_refs
| (selected_candidate_refs($source_refs; $focus_candidate_ids)) as $selected_refs
| ($selected_refs | map(.candidate_id) | unique_sorted) as $selected_candidate_ids
| (selected_historical_index_refs($source_manifest; $carry_historical_index_refs)) as $selected_historical_index_refs
| {
    summary:focused_retest_manifest_summary(
      $generated_at;
      $status_file;
      $source_manifest_file;
      $focus_manifest_output;
      $focus_summary_output;
      $actions;
      $include_historical_index_refs;
      $carry_historical_index_refs;
      $source_manifest;
      $focus_rows;
      $focus_candidate_ids;
      $selected_refs;
      $selected_candidate_ids;
      $selected_historical_index_refs
    ),
    manifest:focused_retest_manifest(
      $source_manifest;
      $focus_packet_id;
      $focus_run_scope;
      $selected_refs;
      $selected_historical_index_refs
    )
  }
