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
  driver_summary_json="$(cat "$DRIVER_SUMMARY_FILE")"
  driver_manifest_summary_file="$(jq -r '.manifest_summary_file // empty' "$DRIVER_SUMMARY_FILE")"
  if [[ -n "$driver_manifest_summary_file" && -f "$driver_manifest_summary_file" ]]; then
    driver_manifest_summary_json="$(cat "$driver_manifest_summary_file")"
  else
    driver_manifest_summary_json="null"
  fi
else
  driver_summary_json="null"
  driver_manifest_summary_json="null"
fi

jq \
  --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --arg plan_file "$PLAN_FILE" \
  --arg driver_summary_file "$DRIVER_SUMMARY_FILE" \
  --argjson driver_summary "$driver_summary_json" \
  --argjson driver_manifest_summary "$driver_manifest_summary_json" \
  '
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

    (.horizon_rows // []) as $rows
    | ($driver_summary // {}) as $driver
    | ($driver_manifest_summary // {}) as $manifest_summary
    | ($driver.manifest.latest_universe // $manifest_summary.latest_universe // {}) as $latest_universe
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
          research_replay_completed:($driver.stage_state.research_replay_completed // null),
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
    | ($latest_universe.observed_symbols // []) as $observed_symbols
    | ($latest_universe.approved_symbols // []) as $approved_symbols
    | ($candidate_symbols | intersect($approved_symbols)) as $candidate_symbols_in_approved_universe
    | ($rows | map(select((.replay_run_count // 0) > 0) | .primary_symbol) | unique_sorted) as $research_replayed_symbols
    | ($rows | map(select(.next_action == "promotion_gate_ready_for_review") | .primary_symbol) | unique_sorted) as $promotion_ready_symbols
    | ($rows | map(select(any((.gate_biases // [])[]?; startswith("PROMOTE"))) | .primary_symbol) | unique_sorted) as $promoted_symbols
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
          candidate_symbols_in_approved_universe:$candidate_symbols_in_approved_universe,
          approved_symbols_without_selected_candidate:($approved_symbols - $candidate_symbols),
          selected_symbols_not_in_approved_universe:(
            if ($approved_symbols | length) == 0 then []
            else ($candidate_symbols - $approved_symbols)
            end
          ),
          candidate_symbol_coverage_of_approved_universe:(
            if ($approved_symbols | length) == 0 then null
            else (($candidate_symbols_in_approved_universe | length) / ($approved_symbols | length))
            end
          )
        },
        research_factory_progression:{
          major50_observed_symbol_count:($latest_universe.observed_symbol_count // ($observed_symbols | length)),
          major50_approved_symbol_count:($latest_universe.approved_symbol_count // ($approved_symbols | length)),
          candidate_generated_symbol_count:($candidate_symbols | length),
          research_replayed_symbol_count:($research_replayed_symbols | length),
          promotion_ready_symbol_count:($promotion_ready_symbols | length),
          promoted_symbol_count:($promoted_symbols | length),
          shadow_created:((.stage_state.shadow_created // false) == true),
          paper_created:((.stage_state.paper_created // false) == true),
          live_enabled:false,
          symbols:{
            candidate_generated:$candidate_symbols,
            research_replayed:$research_replayed_symbols,
            promotion_ready:$promotion_ready_symbols,
            promoted:$promoted_symbols
          }
        }
      }
  ' "$PLAN_FILE"

echo "research retest horizon status summary completed" >&2
