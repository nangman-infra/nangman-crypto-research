include "summarize-retest-horizon-status-lib";

def plan_research_replay_completed($rows):
  ($rows | map(.candidate_id) | unique_sorted) as $all_candidate_ids_for_stage
  | (
      $rows
      | map(select((.replay_run_count // 0) > 0) | .candidate_id)
      | unique_sorted
    ) as $replayed_candidate_ids_for_stage
  | (
      ($all_candidate_ids_for_stage | length) > 0
      and (($all_candidate_ids_for_stage - $replayed_candidate_ids_for_stage) | length) == 0
    );

def candidate_horizon_matrix($rows):
  $rows
  | sort_by(.primary_symbol, .candidate_id, .horizon)
  | group_by(.candidate_id)
  | map(. as $candidate_rows
    | (tracked_horizons | map(candidate_horizon_state($candidate_rows; .))) as $tracked
    | {
        candidate_id:$candidate_rows[0].candidate_id,
        candidate_lifecycle_key:$candidate_rows[0].candidate_lifecycle_key,
        primary_symbol:$candidate_rows[0].primary_symbol,
        symbols:$candidate_rows[0].symbols,
        hypothesis_type:$candidate_rows[0].hypothesis_type,
        research_priority:$candidate_rows[0].research_priority,
        tracked_horizons:$tracked,
        next_action_counts:($tracked | action_counts),
        requested_horizon_count:($tracked | map(select(.requested == true)) | length),
        missing_tracked_horizon_count:($tracked | map(select(.requested != true)) | length),
        promotion_ready_horizon_count:($tracked | map(select(.promotion_gate_ready_for_review == true)) | length)
      })
  | sort_by(.primary_symbol, .candidate_id);

def candidate_horizon_matrix_summary($candidate_horizon_matrix):
  {
    tracked_horizons:tracked_horizons,
    candidate_count:($candidate_horizon_matrix | length),
    requested_horizon_slot_count:([
      $candidate_horizon_matrix[].tracked_horizons[]?
      | select(.requested == true)
    ] | length),
    missing_tracked_horizon_slot_count:([
      $candidate_horizon_matrix[].tracked_horizons[]?
      | select(.requested != true)
    ] | length),
    promotion_ready_horizon_count:([
      $candidate_horizon_matrix[].tracked_horizons[]?
      | select(.promotion_gate_ready_for_review == true)
    ] | length),
    next_action_counts:(
      [$candidate_horizon_matrix[].tracked_horizons[]?]
      | action_counts
    )
  };

def by_symbol_summary($rows):
  $rows
  | sort_by(.primary_symbol, .candidate_id, .horizon)
  | group_by(.primary_symbol)
  | map({
      symbol:.[0].primary_symbol,
      candidate_count:(map(.candidate_id) | unique | length),
      horizon_count:length,
      horizons:horizon_counts,
      next_action_counts:action_counts,
      ready_for_replay_count:(
        count_action("run_research_replay_for_horizon")
        + count_action("materialize_completed_native_replay_sample")
      ),
      waiting_for_market_l1_count:count_action("wait_for_market_l1_horizon"),
      market_l1_coverage_extension_count:count_action("extend_market_l1_horizon_coverage"),
      sample_accumulation_count:count_action("accumulate_completed_native_replay_samples"),
      promotion_ready_for_review_count:count_action("promotion_gate_ready_for_review"),
      candidates:(
        sort_by(.candidate_id, .horizon)
        | group_by(.candidate_id)
        | map({
            candidate_id:.[0].candidate_id,
            candidate_lifecycle_key:.[0].candidate_lifecycle_key,
            symbols:.[0].symbols,
            hypothesis_type:.[0].hypothesis_type,
            research_priority:.[0].research_priority,
            horizons:(. | compact_rows | sort_by(.horizon | horizon_order))
          })
      )
    });
