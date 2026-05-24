#!/usr/bin/env bash
set -euo pipefail

OBSERVATION_PLAN_FILE="${RESEARCH_SHADOW_OBSERVATION_PLAN_FILE:-${1:-}}"

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

require_absolute_file "RESEARCH_SHADOW_OBSERVATION_PLAN_FILE or first argument" "$OBSERVATION_PLAN_FILE"

jq \
  --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --argjson generated_at_ms "$(date -u +%s)000" \
  --arg observation_plan_file "$OBSERVATION_PLAN_FILE" \
  '
    def unique_sorted: unique | sort;
    def count_status($counts; $status):
      ($counts | map(select(.value == $status) | .count) | add) // 0;
    def sample_status:
      .observation_sample_status // {};
    def candidate_projection:
      sample_status as $sample
      | (.status_counts // []) as $status_counts
      | {
          candidate_lifecycle_key,
          symbols:(.symbols // []),
          status_counts:$status_counts,
          pending_count:count_status($status_counts; "pending"),
          completed_count:count_status($status_counts; "completed"),
          failed_count:count_status($status_counts; "failed"),
          target_window_materialized_count:(.target_window_materialized_count // 0),
          absolute_window_materialized_count:(.absolute_window_materialized_count // 0),
          observed_shadow_run_count:($sample.observed_shadow_run_count // 0),
          target_window_materialized_shadow_run_count:($sample.target_window_materialized_shadow_run_count // 0),
          required_shadow_sample_count:($sample.required_shadow_sample_count // 0),
          sample_requirement_basis:($sample.sample_requirement_basis // "target_window_materialized_shadow_run_count"),
          sample_requirement_met:($sample.sample_requirement_met // false),
          sample_deficit:($sample.sample_deficit // 0),
          recommended_action:(
            if (($sample.sample_requirement_met // false) == true) then "review_shadow_completion_evidence"
            elif ((.target_window_materialized_count // 0) == 0) then "wait_for_target_holding_window"
            else "accumulate_shadow_observation_samples" end
          )
        };

    . as $plan
    | (($plan.by_candidate_lifecycle_key // []) | map(candidate_projection)) as $candidates
    | ($candidates | map(select(.sample_deficit > 0))) as $deficient
    | ($candidates | map(select(.sample_requirement_met == true))) as $sample_ready
    | ($candidates | map(select(.target_window_materialized_count == 0))) as $target_waiting
    | ($candidates | map(select(.pending_count > 0))) as $pending
    | {
        schema_version:"research_shadow_sample_gap_manifest_v1",
        generated_at:$generated_at,
        generated_at_ms:$generated_at_ms,
        shadow_observation_plan_file:$observation_plan_file,
        safety:{
          s3_write:false,
          ecs_task_started:false,
          dispatcher_mode_changed:false,
          local_manifest_only:true,
          shadow_status_mutated:false,
          paper_live_enabled:false
        },
        source_state:{
          observation_plan_schema_version:($plan.schema_version // null),
          observation_plan_verdict:($plan.next_decision.verdict // null),
          latest_l1_as_of_ms:($plan.latest_l1_as_of_ms // null),
          latest_l1_source:($plan.latest_l1_source // null),
          shadow_validation_run_file:($plan.shadow_validation_run_file // null),
          retest_horizon_status_file:($plan.retest_horizon_status_file // null)
        },
        shadow_sample_gap_summary:{
          candidate_lifecycle_count:($candidates | length),
          symbol_count:($candidates | map(.symbols // []) | flatten | unique | length),
          symbols:($candidates | map(.symbols // []) | flatten | unique_sorted),
          pending_candidate_count:($pending | length),
          target_window_waiting_candidate_count:($target_waiting | length),
          partially_materialized_candidate_count:(
            $candidates
            | map(select(.target_window_materialized_count > 0 and .target_window_materialized_shadow_run_count < .observed_shadow_run_count))
            | length
          ),
          sample_requirement_met_candidate_count:($sample_ready | length),
          deficient_candidate_count:($deficient | length),
          total_sample_deficit:(($deficient | map(.sample_deficit) | add) // 0),
          largest_sample_deficit:(($deficient | map(.sample_deficit) | max) // 0),
          minimum_required_shadow_sample_count:(($candidates | map(.required_shadow_sample_count) | min) // 0),
          maximum_required_shadow_sample_count:(($candidates | map(.required_shadow_sample_count) | max) // 0)
        },
        next_decision:{
          verdict:(
            if ($candidates | length) == 0 then "NO_SHADOW_CANDIDATES"
            elif ($plan.latest_l1_as_of_ms // null) == null then "DISCOVER_LATEST_MARKET_L1_AS_OF"
            elif ($target_waiting | length) > 0 then "WAIT_FOR_TARGET_HOLDING_WINDOW"
            elif ($deficient | length) > 0 then "ACCUMULATE_SHADOW_SAMPLES_BEFORE_COMPLETION"
            elif ($pending | length) > 0 then "REVIEW_SHADOW_COMPLETION_EVIDENCE"
            else "NO_SHADOW_SAMPLE_GAP_DETECTED" end
          ),
          safe_next_actions:[
            if ($plan.latest_l1_as_of_ms // null) == null then "discover_latest_market_l1_as_of" else empty end,
            if ($target_waiting | length) > 0 then "wait_for_target_holding_window_materialization" else empty end,
            if ($deficient | length) > 0 then "accumulate_shadow_observation_samples" else empty end,
            if ($pending | length) > 0 then "keep_shadow_status_pending_until_completion_evidence_exists" else empty end,
            if ($sample_ready | length) > 0 then "review_sample_ready_candidates_for_shadow_completion" else empty end
          ],
          blocked_actions:[
            "do_not_mark_pending_shadow_passed_from_sample_counts_only",
            "do_not_create_paper_without_completed_passed_shadow",
            "do_not_enable_live_from_shadow_sample_gap_manifest"
          ]
        },
        shadow_sample_backlog:(
          $deficient
          | sort_by(-.sample_deficit, .candidate_lifecycle_key)
        ),
        sample_ready_candidates:(
          $sample_ready
          | sort_by(.candidate_lifecycle_key)
        ),
        by_candidate_lifecycle_key:(
          $candidates
          | sort_by(-.sample_deficit, .candidate_lifecycle_key)
        )
      }
  ' "$OBSERVATION_PLAN_FILE"

echo "research shadow sample gap manifest completed" >&2
