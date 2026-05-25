#!/usr/bin/env bash
set -euo pipefail

STATUS_FILE="${RESEARCH_HORIZON_STATUS_FILE:-${1:-}}"
SOURCE_MANIFEST_FILE="${RESEARCH_SOURCE_MANIFEST_FILE:-${2:-}}"
FOCUS_NEXT_ACTIONS="${RESEARCH_FOCUS_NEXT_ACTIONS:-run_research_replay_for_horizon,accumulate_completed_native_replay_samples,materialize_completed_native_replay_sample}"
FOCUS_PACKET_ID="${RESEARCH_FOCUS_PACKET_ID:-research_focus_$(date -u +%Y%m%dT%H%M%SZ)}"
FOCUS_RUN_SCOPE="${RESEARCH_FOCUS_RUN_SCOPE:-focused_retest_local_validation}"
INCLUDE_HISTORICAL_INDEX_REFS="${RESEARCH_FOCUS_INCLUDE_HISTORICAL_INDEX_REFS:-auto}"
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
require_absolute_file "RESEARCH_HORIZON_STATUS_FILE or first argument" "$STATUS_FILE"
require_absolute_file "RESEARCH_SOURCE_MANIFEST_FILE or second argument" "$SOURCE_MANIFEST_FILE"
case "$INCLUDE_HISTORICAL_INDEX_REFS_NORMALIZED" in
  auto | true | false) ;;
  *)
    echo "RESEARCH_FOCUS_INCLUDE_HISTORICAL_INDEX_REFS must be auto, true, or false; got $INCLUDE_HISTORICAL_INDEX_REFS" >&2
    exit 1
    ;;
esac

if [[ -n "${RESEARCH_FOCUS_MANIFEST_OUTPUT:-}" ]]; then
  FOCUS_MANIFEST_OUTPUT="$RESEARCH_FOCUS_MANIFEST_OUTPUT"
else
  tmp_root="${TMPDIR:-/tmp}"
  tmp_root="${tmp_root%/}"
  FOCUS_MANIFEST_OUTPUT="$(mktemp "${tmp_root}/research-focused-input-manifest.XXXXXX")"
fi
FOCUS_SUMMARY_OUTPUT="${RESEARCH_FOCUS_SUMMARY_OUTPUT:-${FOCUS_MANIFEST_OUTPUT}.summary.json}"
require_absolute_path "RESEARCH_FOCUS_MANIFEST_OUTPUT" "$FOCUS_MANIFEST_OUTPUT"
require_absolute_path "RESEARCH_FOCUS_SUMMARY_OUTPUT" "$FOCUS_SUMMARY_OUTPUT"

summary_tmp="$(mktemp)"
manifest_tmp="$(mktemp)"
trap 'rm -f "$summary_tmp" "$manifest_tmp"' EXIT

jq -n \
  --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --arg status_file "$STATUS_FILE" \
  --arg source_manifest_file "$SOURCE_MANIFEST_FILE" \
  --arg focus_manifest_output "$FOCUS_MANIFEST_OUTPUT" \
  --arg focus_summary_output "$FOCUS_SUMMARY_OUTPUT" \
  --arg focus_next_actions "$FOCUS_NEXT_ACTIONS" \
  --arg focus_packet_id "$FOCUS_PACKET_ID" \
  --arg focus_run_scope "$FOCUS_RUN_SCOPE" \
  --arg include_historical_index_refs "$INCLUDE_HISTORICAL_INDEX_REFS_NORMALIZED" \
  --slurpfile status "$STATUS_FILE" \
  --slurpfile source "$SOURCE_MANIFEST_FILE" \
  '
    def unique_sorted: unique | sort;
    def action_list:
      $focus_next_actions
      | split(",")
      | map(gsub("^\\s+|\\s+$"; ""))
      | map(select(length > 0))
      | unique_sorted;
    def candidate_id_from_uri:
      ((.uri // "" | capture("candidate_id=(?<candidate_id>[^/]+)")? // {}) | .candidate_id) // null;
    def horizon_order:
      if . == "1h" then 1
      elif . == "4h" then 2
      elif . == "24h" or . == "1d" then 3
      elif . == "72h" then 4
      elif . == "7d" then 5
      else 99 end;

    ($status[0]) as $status_doc
    | ($source[0]) as $source_manifest
    | (action_list) as $actions
    | (
        [
          $status_doc.by_symbol[]? as $symbol
          | $symbol.candidates[]? as $candidate
          | $candidate.horizons[]?
          | select(.next_action as $action | ($actions | index($action)) != null)
          | . + {
              focus_symbol:$symbol.symbol,
              candidate_id:($candidate.candidate_id // .candidate_id),
              candidate_lifecycle_key:($candidate.candidate_lifecycle_key // .candidate_lifecycle_key),
              hypothesis_type:($candidate.hypothesis_type // .hypothesis_type),
              research_priority:($candidate.research_priority // .research_priority)
            }
        ]
        | sort_by(.focus_symbol, .candidate_id, (.horizon | horizon_order))
      ) as $focus_rows
    | ($focus_rows | map(.candidate_id) | unique_sorted) as $focus_candidate_ids
    | (
        $include_historical_index_refs == "true"
        or (
          $include_historical_index_refs == "auto"
          and ($actions | index("accumulate_completed_native_replay_samples")) != null
        )
      ) as $carry_historical_index_refs
    | (
        ($source_manifest.candidate_bundle_refs // [])
        | map(. + {candidate_id:candidate_id_from_uri})
      ) as $source_refs
    | (
        $source_refs
        | map(select(.candidate_id as $candidate_id | $candidate_id != null and ($focus_candidate_ids | index($candidate_id)) != null))
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
    | {
        summary:{
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
            next_action_counts:(
              $focus_rows
              | sort_by(.next_action)
              | group_by(.next_action)
              | map({next_action:.[0].next_action, count:length})
              | sort_by(.count, .next_action)
              | reverse
            ),
            horizons:(
              $focus_rows
              | sort_by(.horizon)
              | group_by(.horizon)
              | map({horizon:.[0].horizon, count:length})
              | sort_by(.horizon | horizon_order)
            ),
            selected_candidate_ids:$selected_candidate_ids,
            missing_candidate_ref_ids:($focus_candidate_ids - $selected_candidate_ids),
            rows:(
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
                })
            )
          }
        },
        manifest:{
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
        }
      }
  ' > "$summary_tmp"

jq '.summary' "$summary_tmp" > "$FOCUS_SUMMARY_OUTPUT"
jq '.manifest' "$summary_tmp" > "$FOCUS_MANIFEST_OUTPUT"

selected_count="$(jq -r '.focused.selected_candidate_bundle_ref_count' "$FOCUS_SUMMARY_OUTPUT")"
if [[ "$selected_count" == "0" ]]; then
  jq -r '
    "focus_horizon_count=\(.focused.focus_horizon_count)",
    "focus_candidate_count=\(.focused.focus_candidate_count)",
    "selected_candidate_bundle_ref_count=\(.focused.selected_candidate_bundle_ref_count)",
    "missing_candidate_ref_ids=\(.focused.missing_candidate_ref_ids | join(","))"
  ' "$FOCUS_SUMMARY_OUTPUT"
  echo "no focused candidate bundle refs were selected" >&2
  exit 1
fi

jq -r '
  "focus_manifest_output=\(.focus_manifest_output)",
  "focus_summary_output=\(.focus_summary_output)",
  "focus_next_actions=\(.focus_next_actions | join(","))",
  "focus_horizon_count=\(.focused.focus_horizon_count)",
  "focus_candidate_count=\(.focused.focus_candidate_count)",
  "selected_candidate_bundle_ref_count=\(.focused.selected_candidate_bundle_ref_count)",
  "selected_historical_replay_run_index_ref_count=\(.focused.selected_historical_replay_run_index_ref_count)",
  "symbols=\(.focused.symbols | join(","))",
  "horizons=\(.focused.horizons | map(.horizon + ":" + (.count|tostring)) | join(","))",
  "next_action_counts=\(.focused.next_action_counts | map(.next_action + ":" + (.count|tostring)) | join(","))",
  "safety=s3_write:\(.safety.s3_write),ecs_task_started:\(.safety.ecs_task_started),dispatcher_mode_changed:\(.safety.dispatcher_mode_changed),historical_replay_run_index_refs_carried:\(.safety.historical_replay_run_index_refs_carried)"
' "$FOCUS_SUMMARY_OUTPUT"

echo "focused retest manifest build completed"
