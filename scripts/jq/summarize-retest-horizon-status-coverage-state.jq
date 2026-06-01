include "summarize-retest-horizon-status-lib";

def research_factory_coverage_state($rows; $driver; $latest_universe):
  (.horizon_summary.symbols // []) as $candidate_symbols
  | ($latest_universe.observed_symbols // [] | unique_sorted) as $observed_symbols
  | ($latest_universe.approved_symbols // [] | unique_sorted) as $approved_symbols
  | ($candidate_symbols | intersect($approved_symbols)) as $candidate_symbols_in_approved_universe
  | (($driver.manifest.eligible_candidate_symbols // $candidate_symbols) | unique_sorted) as $eligible_candidate_symbols
  | (($driver.manifest.unselected_eligible_candidate_symbols // []) | unique_sorted) as $unselected_eligible_candidate_symbols
  | ($eligible_candidate_symbols | intersect($approved_symbols)) as $eligible_candidate_symbols_in_approved_universe
  | ($approved_symbols - $candidate_symbols) as $approved_symbols_without_selected_candidate
  | ($approved_symbols - $eligible_candidate_symbols) as $approved_symbols_without_eligible_candidate
  | ($rows | map(select((.replay_run_count // 0) > 0) | .primary_symbol) | unique_sorted) as $research_replayed_symbols
  | ($rows | map(select(.next_action == "promotion_gate_ready_for_review") | .primary_symbol) | unique_sorted) as $promotion_ready_symbols
  | ($rows | map(select(any((.gate_biases // [])[]?; startswith("PROMOTE"))) | .primary_symbol) | unique_sorted) as $promoted_symbols
  | ($rows | map(.candidate_id) | unique_sorted) as $candidate_ids
  | ($rows | map(select((.replay_run_count // 0) > 0) | .candidate_id) | unique_sorted) as $research_replayed_candidate_ids
  | ($rows | map(select(.next_action == "promotion_gate_ready_for_review") | .candidate_id) | unique_sorted) as $promotion_ready_candidate_ids
  | ($rows | map(select(any((.gate_biases // [])[]?; startswith("PROMOTE"))) | .candidate_id) | unique_sorted) as $promoted_candidate_ids
  | ($candidate_symbols - $research_replayed_symbols) as $candidate_symbols_without_replay
  | ($candidate_ids - $research_replayed_candidate_ids) as $candidate_ids_without_replay
  | ($research_replayed_symbols - $promotion_ready_symbols) as $replayed_symbols_without_promotion_ready
  | ($research_replayed_symbols - $promoted_symbols) as $replayed_symbols_without_promotion
  | ($research_replayed_candidate_ids - $promotion_ready_candidate_ids) as $replayed_candidate_ids_without_promotion_ready
  | ($research_replayed_candidate_ids - $promoted_candidate_ids) as $replayed_candidate_ids_without_promotion
  | {
      observed_symbols:$observed_symbols,
      approved_symbols:$approved_symbols,
      candidate_symbols:$candidate_symbols,
      candidate_symbols_in_approved_universe:$candidate_symbols_in_approved_universe,
      eligible_candidate_symbols:$eligible_candidate_symbols,
      unselected_eligible_candidate_symbols:$unselected_eligible_candidate_symbols,
      eligible_candidate_symbols_in_approved_universe:$eligible_candidate_symbols_in_approved_universe,
      approved_symbols_without_selected_candidate:$approved_symbols_without_selected_candidate,
      approved_symbols_without_eligible_candidate:$approved_symbols_without_eligible_candidate,
      research_replayed_symbols:$research_replayed_symbols,
      promotion_ready_symbols:$promotion_ready_symbols,
      promoted_symbols:$promoted_symbols,
      candidate_ids:$candidate_ids,
      research_replayed_candidate_ids:$research_replayed_candidate_ids,
      promotion_ready_candidate_ids:$promotion_ready_candidate_ids,
      promoted_candidate_ids:$promoted_candidate_ids,
      candidate_symbols_without_replay:$candidate_symbols_without_replay,
      candidate_ids_without_replay:$candidate_ids_without_replay,
      replayed_symbols_without_promotion_ready:$replayed_symbols_without_promotion_ready,
      replayed_symbols_without_promotion:$replayed_symbols_without_promotion,
      replayed_candidate_ids_without_promotion_ready:$replayed_candidate_ids_without_promotion_ready,
      replayed_candidate_ids_without_promotion:$replayed_candidate_ids_without_promotion,
      blocking_stage:(
        if (($approved_symbols | length) > 0 and ($approved_symbols_without_eligible_candidate | length) > 0)
          then "candidate_generation_coverage"
        elif (
          (($driver.manifest.selected_candidate_limit_reached // false) == true)
          and (($unselected_eligible_candidate_symbols | length) > 0)
        )
          then "research_manifest_selection_cap"
        elif (($candidate_ids_without_replay | length) > 0)
          then "research_replay_coverage"
        elif (($promotion_ready_symbols | length) > 0 and ((.stage_state.shadow_created // false) != true))
          then "shadow_review_gate"
        elif (($promoted_symbols | length) == 0)
          then "promotion_evidence"
        elif ((.stage_state.paper_created // false) != true)
          then "paper_validation_gate"
        elif ((.stage_state.live_enabled // false) != true)
          then "human_live_approval_boundary"
        else "no_gap_detected"
        end
      )
    };
