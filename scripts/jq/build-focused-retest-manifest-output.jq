include "build-focused-retest-manifest-common";

def next_action_counts($focus_rows):
  $focus_rows
  | sort_by(.next_action)
  | group_by(.next_action)
  | map({next_action:.[0].next_action, count:length})
  | sort_by(.count, .next_action)
  | reverse;

def horizon_counts($focus_rows):
  $focus_rows
  | sort_by(.horizon)
  | group_by(.horizon)
  | map({horizon:.[0].horizon, count:length})
  | sort_by(.horizon | horizon_order);

def focused_retest_rows_for_summary($focus_rows):
  $focus_rows
  | map({
      candidate_id,
      candidate_lifecycle_key,
      symbol:.focus_symbol,
      symbols,
      hypothesis_type,
      research_priority,
      horizon,
      next_action,
      replay_run_count,
      completed_count,
      completed_sample_deficit,
      inferred_unseen_window_count,
      unseen_window_deficit,
      reason_codes
    });

def focused_retest_manifest_summary(
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
):
  {
    schema_version:"research_focused_retest_manifest_summary_v1",
    generated_at:$generated_at,
    status_file:$status_file,
    source_manifest_file:$source_manifest_file,
    focus_manifest_output:$focus_manifest_output,
    focus_summary_output:$focus_summary_output,
    focus_next_actions:$actions,
    safety:{
      s3_write:false,
      ecs_task_started:false,
      dispatcher_mode_changed:false,
      local_manifest_only:true,
      selected_from_existing_current_approved_status:true,
      historical_replay_run_index_ref_mode:$include_historical_index_refs,
      historical_replay_run_index_refs_carried:$carry_historical_index_refs
    },
    source:{
      research_packet_id:$source_manifest.research_packet_id,
      run_scope:$source_manifest.run_scope,
      candidate_bundle_ref_count:(($source_manifest.candidate_bundle_refs // []) | length),
      historical_replay_run_index_ref_count:(($source_manifest.historical_replay_run_index_refs // []) | length)
    },
    focused:{
      focus_horizon_count:($focus_rows | length),
      focus_candidate_count:($focus_candidate_ids | length),
      selected_candidate_bundle_ref_count:($selected_refs | length),
      selected_historical_replay_run_index_ref_count:($selected_historical_index_refs | length),
      symbols:($focus_rows | map(.focus_symbol) | unique_sorted),
      next_action_counts:next_action_counts($focus_rows),
      horizons:horizon_counts($focus_rows),
      selected_candidate_ids:$selected_candidate_ids,
      missing_candidate_ref_ids:($focus_candidate_ids - $selected_candidate_ids),
      rows:focused_retest_rows_for_summary($focus_rows)
    }
  };

def focused_retest_manifest(
  $source_manifest;
  $focus_packet_id;
  $focus_run_scope;
  $selected_refs;
  $selected_historical_index_refs
):
  {
    schema_version:($source_manifest.schema_version // "research_input_manifest_v1"),
    research_packet_id:$focus_packet_id,
    run_scope:$focus_run_scope,
    candidate_bundle_refs:($selected_refs | map({uri})),
    historical_replay_run_index_refs:$selected_historical_index_refs,
    runtime_budget_policy:(
      ($source_manifest.runtime_budget_policy // {})
      + {
          max_candidate_bundle_count:(
            if ($selected_refs | length) > 0 then ($selected_refs | length) else 1 end
          )
        }
    )
  };
