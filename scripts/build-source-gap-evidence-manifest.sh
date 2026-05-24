#!/usr/bin/env bash
set -euo pipefail

DIAGNOSIS_FILE="${RESEARCH_SOURCE_GAP_DIAGNOSIS_FILE:-${1:-}}"
SOURCE_MANIFEST_FILE="${RESEARCH_SOURCE_MANIFEST_FILE:-${2:-}}"
FOCUS_STATUSES="${RESEARCH_SOURCE_GAP_STATUSES:-candidate_evidence_outside_research_batch_selection}"
CANDIDATE_BUCKET="${RESEARCH_SOURCE_GAP_CANDIDATE_S3_BUCKET:-${RESEARCH_CANDIDATE_S3_BUCKET:-}}"
PACKET_ID="${RESEARCH_SOURCE_GAP_PACKET_ID:-research_source_gap_$(date -u +%Y%m%dT%H%M%SZ)}"
RUN_SCOPE="${RESEARCH_SOURCE_GAP_RUN_SCOPE:-source_gap_existing_evidence_local_validation}"
INCLUDE_HISTORICAL_INDEX_REFS="${RESEARCH_SOURCE_GAP_INCLUDE_HISTORICAL_INDEX_REFS:-false}"
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
require_absolute_file "RESEARCH_SOURCE_GAP_DIAGNOSIS_FILE or first argument" "$DIAGNOSIS_FILE"

case "$INCLUDE_HISTORICAL_INDEX_REFS_NORMALIZED" in
  auto | true | false) ;;
  *)
    echo "RESEARCH_SOURCE_GAP_INCLUDE_HISTORICAL_INDEX_REFS must be auto, true, or false; got $INCLUDE_HISTORICAL_INDEX_REFS" >&2
    exit 1
    ;;
esac

if [[ -n "$SOURCE_MANIFEST_FILE" ]]; then
  require_absolute_file "RESEARCH_SOURCE_MANIFEST_FILE or second argument" "$SOURCE_MANIFEST_FILE"
fi

if [[ -n "${RESEARCH_SOURCE_GAP_MANIFEST_OUTPUT:-}" ]]; then
  MANIFEST_OUTPUT="$RESEARCH_SOURCE_GAP_MANIFEST_OUTPUT"
else
  tmp_root="${TMPDIR:-/tmp}"
  tmp_root="${tmp_root%/}"
  MANIFEST_OUTPUT="$(mktemp "${tmp_root}/research-source-gap-input-manifest.XXXXXX")"
fi
SUMMARY_OUTPUT="${RESEARCH_SOURCE_GAP_SUMMARY_OUTPUT:-${MANIFEST_OUTPUT}.summary.json}"
require_absolute_path "RESEARCH_SOURCE_GAP_MANIFEST_OUTPUT" "$MANIFEST_OUTPUT"
require_absolute_path "RESEARCH_SOURCE_GAP_SUMMARY_OUTPUT" "$SUMMARY_OUTPUT"

source_manifest_input="$(mktemp)"
summary_tmp="$(mktemp)"
trap 'rm -f "$source_manifest_input" "$summary_tmp"' EXIT

if [[ -n "$SOURCE_MANIFEST_FILE" ]]; then
  cp "$SOURCE_MANIFEST_FILE" "$source_manifest_input"
  if [[ -z "$CANDIDATE_BUCKET" ]]; then
    CANDIDATE_BUCKET="$(
      jq -r '
        (.candidate_bundle_refs // [])[]?.uri
        | (capture("^s3://(?<bucket>[^/]+)/")? // {})
        | .bucket // empty
      ' "$SOURCE_MANIFEST_FILE" | head -n 1
    )"
  fi
else
  printf '{}\n' > "$source_manifest_input"
fi

jq -n \
  --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --arg diagnosis_file "$DIAGNOSIS_FILE" \
  --arg source_manifest_file "$SOURCE_MANIFEST_FILE" \
  --arg manifest_output "$MANIFEST_OUTPUT" \
  --arg summary_output "$SUMMARY_OUTPUT" \
  --arg focus_statuses "$FOCUS_STATUSES" \
  --arg candidate_bucket "$CANDIDATE_BUCKET" \
  --arg packet_id "$PACKET_ID" \
  --arg run_scope "$RUN_SCOPE" \
  --arg include_historical_index_refs "$INCLUDE_HISTORICAL_INDEX_REFS_NORMALIZED" \
  --slurpfile diagnosis "$DIAGNOSIS_FILE" \
  --slurpfile source "$source_manifest_input" \
  '
    def unique_sorted: unique | sort;
    def csv_list($value):
      $value
      | split(",")
      | map(gsub("^\\s+|\\s+$"; ""))
      | map(select(length > 0))
      | unique_sorted;
    def candidate_id_from_ref:
      ((. | capture("candidate_id=(?<candidate_id>[^/]+)")? // {}) | .candidate_id) // null;
    def normalize_ref($bucket):
      if startswith("s3://") then .
      elif ($bucket | length) > 0 then "s3://" + $bucket + "/" + (ltrimstr("/"))
      else null
      end;
    def evidence_refs:
      (.evidence_contract.evidence_refs // .evidence_contract.sample_evidence_refs // [])
      | map(select(type == "string" and length > 0));
    def default_runtime_budget($selected_count):
      {
        max_candidate_bundle_count:(if $selected_count > 0 then $selected_count else 1 end),
        max_market_artifact_ref_count:2000,
        max_shadow_validation_run_ref_count:10000,
        max_hypothesis_harness_result_ref_count:10000,
        max_oss_adapter_run_ref_count:10000,
        max_historical_replay_run_ref_count:10000,
        max_replay_run_count:20000
      };

    ($diagnosis[0]) as $diagnosis_doc
    | ($source[0] // {}) as $source_manifest
    | (csv_list($focus_statuses)) as $statuses
    | (
        [
          $diagnosis_doc.symbols[]?
          | select(.status as $status | ($statuses | index($status)) != null)
          | . as $symbol
          | evidence_refs[] as $ref
          | {
              symbol:$symbol.symbol,
              status:$symbol.status,
              primary_blocker:($symbol.primary_blocker // null),
              raw_ref:$ref,
              uri:($ref | normalize_ref($candidate_bucket)),
              candidate_id:($ref | candidate_id_from_ref),
              ref_source_field:(
                if (($symbol.evidence_contract.evidence_refs // []) | length) > 0
                then "evidence_refs"
                else "sample_evidence_refs"
                end
              )
            }
        ]
      ) as $raw_refs
    | ($raw_refs | map(select(.uri == null))) as $missing_bucket_refs
    | (
        $raw_refs
        | map(select(.uri != null))
        | unique_by(.uri)
        | sort_by(.symbol, .candidate_id, .uri)
      ) as $selected_refs
    | (
        $include_historical_index_refs == "true"
        or (
          $include_historical_index_refs == "auto"
          and (($source_manifest.historical_replay_run_index_refs // []) | length) > 0
        )
      ) as $carry_historical_index_refs
    | (
        if $carry_historical_index_refs then
          ($source_manifest.historical_replay_run_index_refs // [])
        else
          []
        end
      ) as $selected_historical_index_refs
    | ($selected_refs | map(.candidate_id) | map(select(. != null)) | unique_sorted) as $selected_candidate_ids
    | ($selected_refs | map(.symbol) | unique_sorted) as $selected_symbols
    | {
        summary:{
          schema_version:"research_source_gap_evidence_manifest_summary_v1",
          generated_at:$generated_at,
          diagnosis_file:$diagnosis_file,
          source_manifest_file:(if $source_manifest_file == "" then null else $source_manifest_file end),
          manifest_output:$manifest_output,
          summary_output:$summary_output,
          focus_statuses:$statuses,
          safety:{
            s3_read:false,
            s3_write:false,
            ecs_task_started:false,
            dispatcher_mode_changed:false,
            local_manifest_only:true,
            selected_existing_candidate_evidence_only:true,
            historical_replay_run_index_ref_mode:$include_historical_index_refs,
            historical_replay_run_index_refs_carried:$carry_historical_index_refs
          },
          source:{
            diagnosis_schema_version:($diagnosis_doc.schema_version // null),
            diagnosis_summary:($diagnosis_doc.summary // {}),
            source_research_packet_id:($source_manifest.research_packet_id // null),
            source_run_scope:($source_manifest.run_scope // null),
            source_candidate_bundle_ref_count:(($source_manifest.candidate_bundle_refs // []) | length),
            source_historical_replay_run_index_ref_count:(($source_manifest.historical_replay_run_index_refs // []) | length),
            inferred_candidate_bucket:(if $candidate_bucket == "" then null else $candidate_bucket end)
          },
          selected:{
            selected_symbol_count:($selected_symbols | length),
            selected_symbols:$selected_symbols,
            selected_candidate_count:($selected_candidate_ids | length),
            selected_candidate_ids:$selected_candidate_ids,
            selected_candidate_bundle_ref_count:($selected_refs | length),
            selected_historical_replay_run_index_ref_count:($selected_historical_index_refs | length),
            status_counts:(
              $selected_refs
              | sort_by(.status)
              | group_by(.status)
              | map({status:.[0].status, count:length})
              | sort_by(.count, .status)
              | reverse
            ),
            primary_blocker_counts:(
              $selected_refs
              | map(select(.primary_blocker != null))
              | sort_by(.primary_blocker)
              | group_by(.primary_blocker)
              | map({primary_blocker:.[0].primary_blocker, count:length})
              | sort_by(.count, .primary_blocker)
              | reverse
            ),
            ref_source_fields:($selected_refs | map(.ref_source_field) | unique_sorted),
            refs:(
              $selected_refs
              | map({
                  symbol,
                  status,
                  primary_blocker,
                  candidate_id,
                  uri,
                  ref_source_field
                })
            )
          },
          blocked:{
            missing_candidate_bucket_ref_count:($missing_bucket_refs | length),
            missing_candidate_bucket_refs:(
              $missing_bucket_refs
              | map({symbol,status,raw_ref})
              | .[0:20]
            )
          }
        },
        manifest:{
          schema_version:($source_manifest.schema_version // "research_input_manifest_v1"),
          research_packet_id:$packet_id,
          run_scope:$run_scope,
          candidate_bundle_refs:($selected_refs | map({uri})),
          historical_replay_run_index_refs:$selected_historical_index_refs,
          runtime_budget_policy:(
            default_runtime_budget($selected_refs | length)
            + ($source_manifest.runtime_budget_policy // {})
            + {
                max_candidate_bundle_count:(
                  if ($selected_refs | length) > 0 then ($selected_refs | length) else 1 end
                )
              }
          )
        }
      }
  ' > "$summary_tmp"

jq '.summary' "$summary_tmp" > "$SUMMARY_OUTPUT"
jq '.manifest' "$summary_tmp" > "$MANIFEST_OUTPUT"

missing_bucket_ref_count="$(jq -r '.blocked.missing_candidate_bucket_ref_count' "$SUMMARY_OUTPUT")"
selected_count="$(jq -r '.selected.selected_candidate_bundle_ref_count' "$SUMMARY_OUTPUT")"

if [[ "$missing_bucket_ref_count" != "0" ]]; then
  jq -r '
    "selected_candidate_bundle_ref_count=\(.selected.selected_candidate_bundle_ref_count)",
    "missing_candidate_bucket_ref_count=\(.blocked.missing_candidate_bucket_ref_count)",
    "missing_candidate_bucket_refs=\(.blocked.missing_candidate_bucket_refs | map(.raw_ref) | join(","))"
  ' "$SUMMARY_OUTPUT"
  echo "candidate bucket is required for key-only evidence refs; set RESEARCH_SOURCE_GAP_CANDIDATE_S3_BUCKET or pass a source manifest" >&2
  exit 1
fi

if [[ "$selected_count" == "0" ]]; then
  jq -r '
    "focus_statuses=\(.focus_statuses | join(","))",
    "selected_symbol_count=\(.selected.selected_symbol_count)",
    "selected_candidate_bundle_ref_count=\(.selected.selected_candidate_bundle_ref_count)"
  ' "$SUMMARY_OUTPUT"
  echo "no source-gap candidate evidence refs were selected" >&2
  exit 1
fi

jq -r '
  "source_gap_manifest_output=\(.manifest_output)",
  "source_gap_summary_output=\(.summary_output)",
  "focus_statuses=\(.focus_statuses | join(","))",
  "selected_symbol_count=\(.selected.selected_symbol_count)",
  "selected_symbols=\(.selected.selected_symbols | join(","))",
  "selected_candidate_count=\(.selected.selected_candidate_count)",
  "selected_candidate_bundle_ref_count=\(.selected.selected_candidate_bundle_ref_count)",
  "selected_historical_replay_run_index_ref_count=\(.selected.selected_historical_replay_run_index_ref_count)",
  "ref_source_fields=\(.selected.ref_source_fields | join(","))",
  "safety=s3_read:\(.safety.s3_read),s3_write:\(.safety.s3_write),ecs_task_started:\(.safety.ecs_task_started),dispatcher_mode_changed:\(.safety.dispatcher_mode_changed),historical_replay_run_index_refs_carried:\(.safety.historical_replay_run_index_refs_carried)"
' "$SUMMARY_OUTPUT"

echo "source-gap evidence manifest build completed"
