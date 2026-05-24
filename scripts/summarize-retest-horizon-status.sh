#!/usr/bin/env bash
set -euo pipefail

PLAN_FILE="${RESEARCH_RETEST_HORIZON_PLAN_FILE:-${1:-}}"
DRIVER_SUMMARY_FILE="${RESEARCH_BATCH_DRIVER_SUMMARY_FILE:-${2:-}}"

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
    exit 1
  fi
}

require_absolute_file() {
  local name="$1"
  local path="$2"
  if [[ -z "$path" || "$path" != /* ]]; then
    echo "$name must be an absolute file path" >&2
    exit 1
  fi
  if [[ ! -f "$path" ]]; then
    echo "$name does not exist: $path" >&2
    exit 1
  fi
}

require_command date
require_command jq
require_absolute_file "RESEARCH_RETEST_HORIZON_PLAN_FILE or first argument" "$PLAN_FILE"

if [[ -n "$DRIVER_SUMMARY_FILE" ]]; then
  require_absolute_file "RESEARCH_BATCH_DRIVER_SUMMARY_FILE or second argument" "$DRIVER_SUMMARY_FILE"
  driver_manifest_summary_file="$(jq -r '.manifest_summary_file // empty' "$DRIVER_SUMMARY_FILE")"
  if [[ -n "$driver_manifest_summary_file" && -f "$driver_manifest_summary_file" ]]; then
    DRIVER_MANIFEST_SUMMARY_FILE="$driver_manifest_summary_file"
  else
    DRIVER_MANIFEST_SUMMARY_FILE=""
  fi
else
  DRIVER_MANIFEST_SUMMARY_FILE=""
fi

driver_summary_input="$(mktemp)"
driver_manifest_summary_input="$(mktemp)"
trap 'rm -f "$driver_summary_input" "$driver_manifest_summary_input"' EXIT

if [[ -n "$DRIVER_SUMMARY_FILE" ]]; then
  cp "$DRIVER_SUMMARY_FILE" "$driver_summary_input"
else
  printf 'null\n' > "$driver_summary_input"
fi

if [[ -n "$DRIVER_MANIFEST_SUMMARY_FILE" ]]; then
  cp "$DRIVER_MANIFEST_SUMMARY_FILE" "$driver_manifest_summary_input"
else
  printf 'null\n' > "$driver_manifest_summary_input"
fi

jq \
  --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --arg plan_file "$PLAN_FILE" \
  --arg driver_summary_file "$DRIVER_SUMMARY_FILE" \
  --slurpfile driver_summary_input "$driver_summary_input" \
  --slurpfile driver_manifest_summary_input "$driver_manifest_summary_input" \
  '($driver_summary_input[0] // null) as $driver_summary
  | ($driver_manifest_summary_input[0] // null) as $driver_manifest_summary
  |
    def unique_sorted: unique | sort;
    def intersect($other):
      map(select(. as $value | ($other | index($value)) != null));
    def horizon_order:
      if . == "1h" then 1
      elif . == "4h" then 2
      elif . == "24h" or . == "1d" then 3
      elif . == "7d" then 4
      else 99 end;
    def action_counts:
      sort_by(.next_action)
      | group_by(.next_action)
      | map({next_action:.[0].next_action, count:length})
      | sort_by(.count, .next_action)
      | reverse;
    def count_action($action):
      map(select(.next_action == $action)) | length;
    def horizon_counts:
      sort_by(.horizon, .next_action)
      | group_by(.horizon)
      | map({
          horizon:.[0].horizon,
          horizon_count:length,
          candidate_count:(map(.candidate_id) | unique | length),
          next_action_counts:action_counts,
          waiting_for_market_l1_count:count_action("wait_for_market_l1_horizon"),
          market_l1_coverage_extension_count:count_action("extend_market_l1_horizon_coverage"),
          ready_for_replay_count:(
            count_action("run_research_replay_for_horizon")
            + count_action("materialize_completed_native_replay_sample")
          ),
          sample_accumulation_count:count_action("accumulate_completed_native_replay_samples"),
          promotion_ready_for_review_count:count_action("promotion_gate_ready_for_review"),
          max_completed_sample_deficit:(map(.completed_sample_deficit // 0) | max // 0),
          max_unseen_window_deficit:(map(.unseen_window_deficit // 0) | max // 0)
        })
      | sort_by(.horizon | horizon_order);
    def compact_rows:
      map({
        candidate_id,
        candidate_lifecycle_key,
        primary_symbol,
        symbols,
        hypothesis_type,
        research_priority,
        horizon,
        horizon_market_data_materialized,
        replay_run_count,
        completed_count,
        completed_sample_deficit,
        inferred_unseen_window_count,
        unseen_window_deficit,
        train_validation_split_required,
        train_validation_split_materialized,
        liquidity_filter_required,
        liquidity_filter_materialized_count,
        missing_market_replay_data_count,
        gate_biases,
        reason_codes,
        next_action
      });
    def tracked_horizons: ["1h", "4h", "24h"];
    def candidate_horizon_state($candidate_rows; $horizon):
      ($candidate_rows | map(select(.horizon == $horizon)) | .[0]) as $row
      | if $row == null then
          {
            horizon:$horizon,
            requested:false,
            next_action:"not_requested",
            horizon_market_data_materialized:false,
            replay_run_count:0,
            completed_count:0,
            completed_sample_deficit:null,
            inferred_unseen_window_count:0,
            unseen_window_deficit:null,
            train_validation_split_materialized:false,
            liquidity_filter_materialized_count:0,
            missing_market_replay_data_count:0,
            gate_biases:[],
            reason_codes:["horizon_not_requested_by_candidate_bundle"],
            promotion_gate_ready_for_review:false
          }
        else
          {
            horizon:$horizon,
            requested:true,
            next_action:$row.next_action,
            horizon_market_data_materialized:($row.horizon_market_data_materialized // false),
            replay_run_count:($row.replay_run_count // 0),
            completed_count:($row.completed_count // 0),
            completed_sample_deficit:($row.completed_sample_deficit // null),
            inferred_unseen_window_count:($row.inferred_unseen_window_count // 0),
            unseen_window_deficit:($row.unseen_window_deficit // null),
            train_validation_split_materialized:($row.train_validation_split_materialized // false),
            liquidity_filter_materialized_count:($row.liquidity_filter_materialized_count // 0),
            missing_market_replay_data_count:($row.missing_market_replay_data_count // 0),
            gate_biases:($row.gate_biases // []),
            reason_codes:($row.reason_codes // []),
            promotion_gate_ready_for_review:($row.next_action == "promotion_gate_ready_for_review")
          }
        end;

    (.horizon_rows // []) as $rows
    | ($driver_summary // {}) as $driver
    | ($driver_manifest_summary // {}) as $manifest_summary
    | ($driver.manifest.latest_universe // $manifest_summary.latest_universe // {}) as $latest_universe
    | ($rows | map(.candidate_id) | unique_sorted) as $all_candidate_ids_for_stage
    | (
        $rows
        | map(select((.replay_run_count // 0) > 0) | .candidate_id)
        | unique_sorted
      ) as $replayed_candidate_ids_for_stage
    | (
        ($all_candidate_ids_for_stage | length) > 0
        and (($all_candidate_ids_for_stage - $replayed_candidate_ids_for_stage) | length) == 0
      ) as $plan_research_replay_completed
    | (
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
        | sort_by(.primary_symbol, .candidate_id)
      ) as $candidate_horizon_matrix
    | {
        schema_version:"research_horizon_status_checkpoint_v1",
        generated_at:$generated_at,
        retest_horizon_plan_file:$plan_file,
        driver_summary_file:(if $driver_summary_file == "" then null else $driver_summary_file end),
        safety:{
          s3_write:false,
          ecs_task_started:false,
          dispatcher_mode_changed:false,
          local_summary_only:true,
          shadow_paper_live_enabled:false
        },
        stage_state:{
          candidate_generated:(
            ($driver.stage_state.candidate_generated // false)
            or (($rows | length) > 0)
          ),
          research_replay_completed:(
            $driver.stage_state.research_replay_completed // $plan_research_replay_completed
          ),
          promotion_passed:($driver.stage_state.promotion_passed // false),
          shadow_created:($driver.stage_state.shadow_created // false),
          paper_created:($driver.stage_state.paper_created // false),
          live_enabled:false
        },
        batch_state:{
          run_id:($driver.run_id // null),
          universe_mode:($driver.manifest.universe_mode // null),
          dispatch_mode:($driver.manifest.dispatch_mode // null),
          selected_candidate_count:($driver.manifest.selected_candidate_count // null),
          eligible_candidate_pool_count:($driver.manifest.eligible_candidate_pool_count // null),
          selected_candidate_limit_reached:($driver.manifest.selected_candidate_limit_reached // null),
          unselected_eligible_candidate_count:($driver.manifest.unselected_eligible_candidate_count // null),
          selected_current_approved_candidate_count:($driver.manifest.selected_current_approved_candidate_count // null),
          research_report_status:($driver.report.research_run_status // null),
          source_candidate_count:($driver.report.source_candidate_count // null),
          replay_run_count:($driver.report.replay_run_count // null),
          retest_candidate_count:($driver.report.retest_candidate_count // null),
          surviving_candidate_count:($driver.report.surviving_candidate_count // null),
          shadow_validation_count:($driver.report.shadow_validation_count // null),
          paper_trade_candidate_count:($driver.report.paper_trade_candidate_count // null)
        },
        horizon_summary:{
          candidate_count:(($rows | map(.candidate_id) | unique) | length),
          horizon_count:($rows | length),
          symbols:($rows | map(.primary_symbol) | unique_sorted),
          next_action_counts:($rows | action_counts),
          ready_for_replay_count:(
            ($rows | count_action("run_research_replay_for_horizon"))
            + ($rows | count_action("materialize_completed_native_replay_sample"))
          ),
          waiting_for_market_l1_count:($rows | count_action("wait_for_market_l1_horizon")),
          market_l1_coverage_extension_count:($rows | count_action("extend_market_l1_horizon_coverage")),
          sample_accumulation_count:($rows | count_action("accumulate_completed_native_replay_samples")),
          promotion_ready_for_review_count:($rows | count_action("promotion_gate_ready_for_review"))
        },
        by_symbol:(
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
            })
        ),
        by_horizon:($rows | horizon_counts),
        candidate_horizon_matrix_summary:{
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
        },
        candidate_horizon_matrix:$candidate_horizon_matrix,
        next_decision:{
          verdict:(
            if (($driver.stage_state.promotion_passed // false) == true) then "PROMOTE_PRESENT_REVIEW_BEFORE_SHADOW"
            elif ($rows | count_action("promotion_gate_ready_for_review")) > 0 then "PROMOTION_GATE_READY_FOR_REVIEW"
            elif ($rows | count_action("extend_market_l1_horizon_coverage")) > 0 then "EXTEND_MARKET_L1_HORIZON_COVERAGE"
            elif (
              (($rows | count_action("run_research_replay_for_horizon"))
              + ($rows | count_action("materialize_completed_native_replay_sample"))) > 0
            ) then "REPLAY_READY_FOR_SOME_HORIZONS"
            elif ($rows | count_action("wait_for_market_l1_horizon")) > 0 then "WAIT_FOR_MARKET_L1_HORIZON"
            elif ($rows | count_action("accumulate_completed_native_replay_samples")) > 0 then "ACCUMULATE_COMPLETED_NATIVE_REPLAY_SAMPLES"
            else "INSPECT_REMAINING_GATE_REASONS" end
          ),
          safe_next_actions:[
            if (($driver.stage_state.promotion_passed // false) == true)
              then "review_promoted_candidates_before_shadow"
              else empty end,
            if (($rows | count_action("promotion_gate_ready_for_review")) > 0)
              then "review_promotion_gate_ready_horizons"
              else empty end,
            if (($rows | count_action("extend_market_l1_horizon_coverage")) > 0)
              then "extend_market_l1_horizon_coverage"
              else empty end,
            if (
              (($rows | count_action("run_research_replay_for_horizon"))
              + ($rows | count_action("materialize_completed_native_replay_sample"))) > 0
            ) then "rerun_current_approved_research_batch_after_market_l1_advances"
              else empty end,
            if (($rows | count_action("wait_for_market_l1_horizon")) > 0)
              then "wait_for_market_l1_horizon_materialization"
              else empty end,
            if (($rows | count_action("accumulate_completed_native_replay_samples")) > 0)
              then "keep_accumulating_completed_native_replay_samples"
              else empty end
          ],
          blocked_actions:[
            if (($driver.stage_state.promotion_passed // false) != true)
              then "do_not_create_shadow_without_promotion"
              else empty end,
            if (($driver.stage_state.shadow_created // false) != true)
              then "do_not_create_paper_without_passed_shadow"
              else empty end,
            "do_not_enable_live_from_research_batch"
          ]
        }
      }
    | (.horizon_summary.symbols // []) as $candidate_symbols
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
    | (
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
      ) as $blocking_stage
    | . + {
        verdict:.next_decision.verdict,
        selected_symbols:$candidate_symbols,
        next_action_counts:.horizon_summary.next_action_counts,
        major50_state:{
          universe_mode:($driver.manifest.universe_mode // null),
          latest_universe_present:($latest_universe.present // null),
          observed_symbol_count:($latest_universe.observed_symbol_count // ($observed_symbols | length)),
          approved_symbol_count:($latest_universe.approved_symbol_count // ($approved_symbols | length)),
          excluded_symbol_count:($latest_universe.excluded_symbol_count // null),
          candidate_symbol_count:($candidate_symbols | length),
          candidate_symbols:$candidate_symbols,
          eligible_candidate_pool_count:($driver.manifest.eligible_candidate_pool_count // null),
          selected_candidate_limit_reached:($driver.manifest.selected_candidate_limit_reached // null),
          unselected_eligible_candidate_count:($driver.manifest.unselected_eligible_candidate_count // null),
          eligible_candidate_symbols:$eligible_candidate_symbols,
          unselected_eligible_candidate_symbols:$unselected_eligible_candidate_symbols,
          candidate_symbols_in_approved_universe:$candidate_symbols_in_approved_universe,
          eligible_candidate_symbols_in_approved_universe:$eligible_candidate_symbols_in_approved_universe,
          approved_symbols_without_selected_candidate:$approved_symbols_without_selected_candidate,
          approved_symbols_without_eligible_candidate:$approved_symbols_without_eligible_candidate,
          selected_symbols_not_in_approved_universe:(
            if ($approved_symbols | length) == 0 then []
            else ($candidate_symbols - $approved_symbols)
            end
          ),
          candidate_symbol_coverage_of_approved_universe:(
            if ($approved_symbols | length) == 0 then null
            else (($candidate_symbols_in_approved_universe | length) / ($approved_symbols | length))
            end
          ),
          eligible_candidate_symbol_coverage_of_approved_universe:(
            if ($approved_symbols | length) == 0 then null
            else (($eligible_candidate_symbols_in_approved_universe | length) / ($approved_symbols | length))
            end
          )
        },
        research_factory_progression:{
          major50_observed_symbol_count:($latest_universe.observed_symbol_count // ($observed_symbols | length)),
          major50_approved_symbol_count:($latest_universe.approved_symbol_count // ($approved_symbols | length)),
          candidate_generated_symbol_count:($candidate_symbols | length),
          candidate_generated_candidate_count:($candidate_ids | length),
          research_replayed_symbol_count:($research_replayed_symbols | length),
          research_replayed_candidate_count:($research_replayed_candidate_ids | length),
          promotion_ready_symbol_count:($promotion_ready_symbols | length),
          promotion_ready_candidate_count:($promotion_ready_candidate_ids | length),
          promoted_symbol_count:($promoted_symbols | length),
          promoted_candidate_count:($promoted_candidate_ids | length),
          shadow_created:((.stage_state.shadow_created // false) == true),
          paper_created:((.stage_state.paper_created // false) == true),
          live_enabled:false,
          symbols:{
            candidate_generated:$candidate_symbols,
            research_replayed:$research_replayed_symbols,
            promotion_ready:$promotion_ready_symbols,
            promoted:$promoted_symbols
          },
          candidates:{
            candidate_generated:$candidate_ids,
            research_replayed:$research_replayed_candidate_ids,
            promotion_ready:$promotion_ready_candidate_ids,
            promoted:$promoted_candidate_ids
          }
        },
        coverage_gaps:{
          approved_symbols_without_candidate:$approved_symbols_without_eligible_candidate,
          approved_symbols_without_selected_candidate:$approved_symbols_without_selected_candidate,
          approved_symbols_without_eligible_candidate:$approved_symbols_without_eligible_candidate,
          unselected_eligible_candidate_symbols:$unselected_eligible_candidate_symbols,
          candidate_symbols_without_replay:$candidate_symbols_without_replay,
          candidate_ids_without_replay:$candidate_ids_without_replay,
          replayed_symbols_without_promotion_ready:$replayed_symbols_without_promotion_ready,
          replayed_symbols_without_promotion:$replayed_symbols_without_promotion,
          replayed_candidate_ids_without_promotion_ready:$replayed_candidate_ids_without_promotion_ready,
          replayed_candidate_ids_without_promotion:$replayed_candidate_ids_without_promotion,
          promotion_ready_symbols_without_shadow:(
            if ((.stage_state.shadow_created // false) == true) then []
            else $promotion_ready_symbols
            end
          ),
          promotion_ready_candidate_ids_without_shadow:(
            if ((.stage_state.shadow_created // false) == true) then []
            else $promotion_ready_candidate_ids
            end
          ),
          promoted_symbols_without_shadow:(
            if ((.stage_state.shadow_created // false) == true) then []
            else $promoted_symbols
            end
          ),
          promoted_candidate_ids_without_shadow:(
            if ((.stage_state.shadow_created // false) == true) then []
            else $promoted_candidate_ids
            end
          )
        },
        research_factory_gap_summary:{
          blocking_stage:$blocking_stage,
          stage_counts:{
            major50_observed:($latest_universe.observed_symbol_count // ($observed_symbols | length)),
            major50_approved:($latest_universe.approved_symbol_count // ($approved_symbols | length)),
            candidate_generated:($candidate_symbols | length),
            candidate_generated_candidates:($candidate_ids | length),
            research_replayed:($research_replayed_symbols | length),
            research_replayed_candidates:($research_replayed_candidate_ids | length),
            promotion_ready:($promotion_ready_symbols | length),
            promotion_ready_candidates:($promotion_ready_candidate_ids | length),
            promoted:($promoted_symbols | length),
            promoted_candidates:($promoted_candidate_ids | length)
          },
          gap_counts:{
            approved_symbols_without_candidate:($approved_symbols_without_eligible_candidate | length),
            approved_symbols_without_selected_candidate:($approved_symbols_without_selected_candidate | length),
            approved_symbols_without_eligible_candidate:($approved_symbols_without_eligible_candidate | length),
            unselected_eligible_candidate_symbols:($unselected_eligible_candidate_symbols | length),
            candidate_symbols_without_replay:($candidate_symbols_without_replay | length),
            candidate_ids_without_replay:($candidate_ids_without_replay | length),
            replayed_symbols_without_promotion_ready:($replayed_symbols_without_promotion_ready | length),
            replayed_symbols_without_promotion:($replayed_symbols_without_promotion | length),
            replayed_candidate_ids_without_promotion_ready:($replayed_candidate_ids_without_promotion_ready | length),
            replayed_candidate_ids_without_promotion:($replayed_candidate_ids_without_promotion | length)
          },
          safe_next_actions:(
            [
              if ($approved_symbols_without_eligible_candidate | length) > 0
                then "increase_candidate_generation_for_approved_major50_symbols"
                else empty end,
              if (($driver.manifest.selected_candidate_limit_reached // false) == true and ($unselected_eligible_candidate_symbols | length) > 0)
                then "increase_research_batch_selection_limit_or_run_focused_manifest"
                else empty end,
              if ($candidate_ids_without_replay | length) > 0
                then "build_focused_research_manifest_for_unreplayed_candidate_symbols"
                else empty end,
              if (.next_decision.safe_next_actions // [] | index("extend_market_l1_horizon_coverage") != null)
                then "extend_market_l1_horizon_coverage_for_current_retest_symbols"
                else empty end,
              if (.next_decision.safe_next_actions // [] | index("wait_for_market_l1_horizon_materialization") != null)
                then "wait_for_market_l1_horizon_materialization"
                else empty end,
              if (.next_decision.safe_next_actions // [] | index("keep_accumulating_completed_native_replay_samples") != null)
                then "keep_accumulating_completed_native_replay_samples"
                else empty end
            ]
            | unique
          )
        }
      }
  ' "$PLAN_FILE"

echo "research retest horizon status summary completed" >&2
