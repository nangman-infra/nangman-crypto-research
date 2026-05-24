#!/usr/bin/env bash
set -euo pipefail

GAP_MANIFEST_FILE="${RESEARCH_SHADOW_SAMPLE_GAP_MANIFEST_FILE:-${1:-}}"
HORIZON_STATUS_FILE="${RESEARCH_RETEST_HORIZON_STATUS_FILE:-${2:-}}"
SOURCE_MANIFEST_FILE="${RESEARCH_SOURCE_MANIFEST_FILE:-${3:-}}"
ACCUMULATION_PACKET_ID="${RESEARCH_SHADOW_ACCUMULATION_PACKET_ID:-research_shadow_accumulation_$(date -u +%Y%m%dT%H%M%SZ)}"
ACCUMULATION_RUN_SCOPE="${RESEARCH_SHADOW_ACCUMULATION_RUN_SCOPE:-shadow_sample_accumulation_local_validation}"
INCLUDE_HISTORICAL_INDEX_REFS="${RESEARCH_SHADOW_ACCUMULATION_INCLUDE_HISTORICAL_INDEX_REFS:-true}"
INCLUDE_HISTORICAL_INDEX_REFS_NORMALIZED="$(printf '%s' "$INCLUDE_HISTORICAL_INDEX_REFS" | tr '[:upper:]' '[:lower:]')"

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

require_absolute_path() {
  local name="$1"
  local path="$2"
  case "$path" in
    /*) ;;
    *)
      echo "$name must be an absolute path; got $path" >&2
      exit 1
      ;;
  esac
}

require_command date
require_command jq
require_command mktemp

require_absolute_file "RESEARCH_SHADOW_SAMPLE_GAP_MANIFEST_FILE or first argument" "$GAP_MANIFEST_FILE"
require_absolute_file "RESEARCH_RETEST_HORIZON_STATUS_FILE or second argument" "$HORIZON_STATUS_FILE"
require_absolute_file "RESEARCH_SOURCE_MANIFEST_FILE or third argument" "$SOURCE_MANIFEST_FILE"
case "$INCLUDE_HISTORICAL_INDEX_REFS_NORMALIZED" in
  true | false) ;;
  *)
    echo "RESEARCH_SHADOW_ACCUMULATION_INCLUDE_HISTORICAL_INDEX_REFS must be true or false; got $INCLUDE_HISTORICAL_INDEX_REFS" >&2
    exit 1
    ;;
esac

if [[ -n "${RESEARCH_SHADOW_ACCUMULATION_MANIFEST_OUTPUT:-}" ]]; then
  ACCUMULATION_MANIFEST_OUTPUT="$RESEARCH_SHADOW_ACCUMULATION_MANIFEST_OUTPUT"
else
  tmp_root="${TMPDIR:-/tmp}"
  tmp_root="${tmp_root%/}"
  ACCUMULATION_MANIFEST_OUTPUT="$(mktemp "${tmp_root}/research-shadow-accumulation-manifest.XXXXXX")"
fi
ACCUMULATION_SUMMARY_OUTPUT="${RESEARCH_SHADOW_ACCUMULATION_SUMMARY_OUTPUT:-${ACCUMULATION_MANIFEST_OUTPUT}.summary.json}"
require_absolute_path "RESEARCH_SHADOW_ACCUMULATION_MANIFEST_OUTPUT" "$ACCUMULATION_MANIFEST_OUTPUT"
require_absolute_path "RESEARCH_SHADOW_ACCUMULATION_SUMMARY_OUTPUT" "$ACCUMULATION_SUMMARY_OUTPUT"

summary_tmp="$(mktemp)"
trap 'rm -f "$summary_tmp"' EXIT

jq -n \
  --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --arg generated_at_ms "$(date -u +%s)000" \
  --arg gap_manifest_file "$GAP_MANIFEST_FILE" \
  --arg horizon_status_file "$HORIZON_STATUS_FILE" \
  --arg source_manifest_file "$SOURCE_MANIFEST_FILE" \
  --arg accumulation_manifest_output "$ACCUMULATION_MANIFEST_OUTPUT" \
  --arg accumulation_summary_output "$ACCUMULATION_SUMMARY_OUTPUT" \
  --arg accumulation_packet_id "$ACCUMULATION_PACKET_ID" \
  --arg accumulation_run_scope "$ACCUMULATION_RUN_SCOPE" \
  --arg include_historical_index_refs "$INCLUDE_HISTORICAL_INDEX_REFS_NORMALIZED" \
  --slurpfile gap "$GAP_MANIFEST_FILE" \
  --slurpfile status "$HORIZON_STATUS_FILE" \
  --slurpfile source "$SOURCE_MANIFEST_FILE" \
  '
    def unique_sorted: unique | sort;
    def candidate_id_from_uri:
      ((.uri // "" | capture("candidate_id=(?<candidate_id>[^/]+)")? // {}) | .candidate_id) // null;
    def count_status($counts; $status):
      ($counts | map(select(.value == $status) | .count) | add) // 0;

    ($gap[0]) as $gap_doc
    | ($status[0]) as $status_doc
    | ($source[0]) as $source_manifest
    | ($include_historical_index_refs == "true") as $carry_historical_index_refs
    | (
        ($gap_doc.shadow_sample_backlog // [])
        | map(select((.sample_deficit // 0) > 0))
      ) as $backlog
    | ($backlog | map(.candidate_lifecycle_key) | unique_sorted) as $backlog_lifecycle_keys
    | (
        [
          $status_doc.candidate_horizon_matrix[]?
          | select(.candidate_lifecycle_key as $key | $key != null and ($backlog_lifecycle_keys | index($key)) != null)
        ]
        | unique_by(.candidate_id)
        | sort_by(.primary_symbol, .candidate_lifecycle_key, .candidate_id)
      ) as $status_candidates
    | ($status_candidates | map(.candidate_id) | unique_sorted) as $status_candidate_ids
    | (
        ($source_manifest.candidate_bundle_refs // [])
        | map(. + {candidate_id:candidate_id_from_uri})
      ) as $source_refs
    | (
        $source_refs
        | map(select(.candidate_id as $id | $id != null and ($status_candidate_ids | index($id)) != null))
        | unique_by(.uri)
      ) as $selected_refs
    | ($selected_refs | map(.candidate_id) | unique_sorted) as $selected_candidate_ids
    | (
        if $carry_historical_index_refs then
          ($source_manifest.historical_replay_run_index_refs // [])
        else
          []
        end
      ) as $selected_historical_index_refs
    | (
        $backlog
        | map(. as $row | {
            candidate_lifecycle_key:$row.candidate_lifecycle_key,
            symbols:($row.symbols // []),
            observed_shadow_run_count:($row.observed_shadow_run_count // 0),
            required_shadow_sample_count:($row.required_shadow_sample_count // 0),
            sample_deficit:($row.sample_deficit // 0),
            pending_count:($row.pending_count // 0),
            status_counts:($row.status_counts // []),
            mapped_candidate_count:(
              $status_candidates
              | map(select(.candidate_lifecycle_key == $row.candidate_lifecycle_key))
              | length
            ),
            selected_candidate_ref_count:(
              $status_candidates
              | map(select(.candidate_lifecycle_key == $row.candidate_lifecycle_key) | .candidate_id)
              | unique
              | map(select(. as $id | $selected_candidate_ids | index($id)))
              | length
            )
          })
        | sort_by(-.sample_deficit, .candidate_lifecycle_key)
      ) as $backlog_projection
    | {
        summary:{
          schema_version:"research_shadow_sample_accumulation_manifest_summary_v1",
          generated_at:$generated_at,
          generated_at_ms:($generated_at_ms | tonumber),
          shadow_sample_gap_manifest_file:$gap_manifest_file,
          retest_horizon_status_file:$horizon_status_file,
          source_manifest_file:$source_manifest_file,
          accumulation_manifest_output:$accumulation_manifest_output,
          accumulation_summary_output:$accumulation_summary_output,
          safety:{
            s3_write:false,
            ecs_task_started:false,
            dispatcher_mode_changed:false,
            local_manifest_only:true,
            shadow_status_mutated:false,
            paper_live_enabled:false,
            selected_from_existing_source_manifest:true,
            historical_replay_run_index_refs_carried:$carry_historical_index_refs
          },
          source_state:{
            gap_manifest_schema_version:($gap_doc.schema_version // null),
            gap_manifest_verdict:($gap_doc.next_decision.verdict // null),
            retest_horizon_status_schema_version:($status_doc.schema_version // null),
            retest_horizon_verdict:($status_doc.verdict // null),
            source_research_packet_id:($source_manifest.research_packet_id // null),
            source_run_scope:($source_manifest.run_scope // null),
            source_candidate_bundle_ref_count:(($source_manifest.candidate_bundle_refs // []) | length),
            source_historical_replay_run_index_ref_count:(($source_manifest.historical_replay_run_index_refs // []) | length)
          },
          backlog_summary:{
            backlog_candidate_lifecycle_count:($backlog_lifecycle_keys | length),
            backlog_symbol_count:($backlog | map(.symbols // []) | flatten | unique | length),
            backlog_symbols:($backlog | map(.symbols // []) | flatten | unique_sorted),
            total_sample_deficit:(($backlog | map(.sample_deficit // 0) | add) // 0),
            largest_sample_deficit:(($backlog | map(.sample_deficit // 0) | max) // 0),
            pending_lifecycle_count:($backlog | map(select((.pending_count // 0) > 0)) | length),
            status_candidate_count:($status_candidates | length),
            selected_candidate_bundle_ref_count:($selected_refs | length),
            selected_historical_replay_run_index_ref_count:($selected_historical_index_refs | length),
            missing_candidate_ref_count:(($status_candidate_ids - $selected_candidate_ids) | length)
          },
          next_decision:{
            verdict:(
              if ($backlog | length) == 0 then "NO_SHADOW_SAMPLE_BACKLOG"
              elif ($status_candidates | length) == 0 then "NO_STATUS_CANDIDATES_FOR_BACKLOG"
              elif ($selected_refs | length) == 0 then "NO_SOURCE_MANIFEST_REFS_FOR_BACKLOG"
              else "RUN_FOCUSED_SHADOW_SAMPLE_ACCUMULATION_RESEARCH" end
            ),
            safe_next_actions:[
              if ($selected_refs | length) > 0 then "run_research_with_shadow_accumulation_manifest" else empty end,
              if ($selected_refs | length) > 0 then "recompute_shadow_observation_plan_after_research" else empty end,
              if ($selected_refs | length) > 0 then "recompute_shadow_sample_gap_manifest_after_research" else empty end,
              if (($status_candidate_ids - $selected_candidate_ids) | length) > 0 then "inspect_missing_candidate_bundle_refs" else empty end
            ],
            blocked_actions:[
              "do_not_mark_pending_shadow_passed_from_accumulation_manifest",
              "do_not_create_paper_without_completed_passed_shadow",
              "do_not_enable_live_from_shadow_accumulation_manifest"
            ]
          },
          shadow_sample_backlog:$backlog_projection,
          selected_candidate_ids:$selected_candidate_ids,
          missing_candidate_ref_ids:($status_candidate_ids - $selected_candidate_ids),
          by_symbol:(
            $status_candidates
            | group_by(.primary_symbol // "unknown")
            | map({
                symbol:.[0].primary_symbol,
                candidate_lifecycle_keys:(map(.candidate_lifecycle_key) | unique_sorted),
                status_candidate_count:length,
                selected_candidate_ref_count:(
                  map(.candidate_id)
                  | unique
                  | map(select(. as $id | $selected_candidate_ids | index($id)))
                  | length
                )
              })
            | sort_by(.symbol)
          )
        },
        manifest:{
          schema_version:($source_manifest.schema_version // "research_input_manifest_v1"),
          research_packet_id:$accumulation_packet_id,
          run_scope:$accumulation_run_scope,
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
        }
      }
  ' > "$summary_tmp"

jq '.summary' "$summary_tmp" > "$ACCUMULATION_SUMMARY_OUTPUT"
jq '.manifest' "$summary_tmp" > "$ACCUMULATION_MANIFEST_OUTPUT"

selected_count="$(jq -r '.backlog_summary.selected_candidate_bundle_ref_count' "$ACCUMULATION_SUMMARY_OUTPUT")"
if [[ "$selected_count" == "0" ]]; then
  jq -r '
    "backlog_candidate_lifecycle_count=\(.backlog_summary.backlog_candidate_lifecycle_count)",
    "status_candidate_count=\(.backlog_summary.status_candidate_count)",
    "selected_candidate_bundle_ref_count=\(.backlog_summary.selected_candidate_bundle_ref_count)",
    "missing_candidate_ref_count=\(.backlog_summary.missing_candidate_ref_count)",
    "verdict=\(.next_decision.verdict)"
  ' "$ACCUMULATION_SUMMARY_OUTPUT"
  echo "no shadow accumulation candidate bundle refs were selected" >&2
  exit 1
fi

jq -r '
  "accumulation_manifest_output=\(.accumulation_manifest_output)",
  "accumulation_summary_output=\(.accumulation_summary_output)",
  "verdict=\(.next_decision.verdict)",
  "backlog_candidate_lifecycle_count=\(.backlog_summary.backlog_candidate_lifecycle_count)",
  "backlog_symbols=\(.backlog_summary.backlog_symbols | join(","))",
  "total_sample_deficit=\(.backlog_summary.total_sample_deficit)",
  "status_candidate_count=\(.backlog_summary.status_candidate_count)",
  "selected_candidate_bundle_ref_count=\(.backlog_summary.selected_candidate_bundle_ref_count)",
  "selected_historical_replay_run_index_ref_count=\(.backlog_summary.selected_historical_replay_run_index_ref_count)",
  "missing_candidate_ref_count=\(.backlog_summary.missing_candidate_ref_count)",
  "safety=s3_write:\(.safety.s3_write),ecs_task_started:\(.safety.ecs_task_started),dispatcher_mode_changed:\(.safety.dispatcher_mode_changed),shadow_status_mutated:\(.safety.shadow_status_mutated),paper_live_enabled:\(.safety.paper_live_enabled)"
' "$ACCUMULATION_SUMMARY_OUTPUT"

echo "shadow sample accumulation manifest build completed"
