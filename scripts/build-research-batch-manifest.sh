#!/usr/bin/env bash
set -euo pipefail

APP_NAME="research-app"
REGION="${AWS_REGION:-${AWS_DEFAULT_REGION:-ap-northeast-2}}"
DISPATCHER_FUNCTION="${RESEARCH_DISPATCHER_FUNCTION:-lmbd-nangman-dev-research-apn2}"
TASK_DEFINITION="${RESEARCH_ECS_TASK_DEFINITION:-td-nangman-dev-research-apn2}"
CONTAINER_NAME="${RESEARCH_ECS_CONTAINER:-research-app}"
CANDIDATE_READ_LIMIT="${RESEARCH_BATCH_CANDIDATE_READ_LIMIT:-1000}"
MAX_CANDIDATE_BUNDLE_COUNT="${RESEARCH_BATCH_MAX_CANDIDATE_BUNDLE_COUNT:-1000}"
HISTORICAL_INDEX_READ_LIMIT="${RESEARCH_BATCH_HISTORICAL_INDEX_READ_LIMIT:-20}"
MAX_HISTORICAL_REPLAY_RUN_REF_COUNT="${RESEARCH_BATCH_MAX_HISTORICAL_REPLAY_RUN_REF_COUNT:-10000}"
MAX_REPLAY_RUN_COUNT="${RESEARCH_BATCH_MAX_REPLAY_RUN_COUNT:-20000}"
UNIVERSE_MODE="${RESEARCH_BATCH_UNIVERSE_MODE:-current_approved}"
RUN_SCOPE="${RESEARCH_BATCH_RUN_SCOPE:-recent_candidate_batch_${UNIVERSE_MODE}_local_validation}"
RESEARCH_PACKET_ID="${RESEARCH_BATCH_PACKET_ID:-research_packet_$(date -u +%Y%m%dT%H%M%SZ)}"

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
    exit 1
  fi
}

redact() {
  sed -E \
    -e 's/nangman-crypto-dev-[A-Za-z0-9-]+-[0-9]{6}/nangman-crypto-dev-<bucket-family>-<account-suffix>/g' \
    -e 's/[0-9]{12}\.dkr\.ecr/<aws-account-id>.dkr.ecr/g' \
    -e 's/account=[0-9]{12}/account=<aws-account-id>/g' \
    -e 's/"Account"[[:space:]]*:[[:space:]]*"[0-9]{12}"/"Account":"<aws-account-id>"/g' \
    -e 's#arn:aws:iam::[^[:space:]"]+#arn:aws:iam::<aws-account-id>:<resource>#g' \
    -e 's#arn:aws:ecs:[^[:space:]"]+#arn:aws:ecs:<region>:<aws-account-id>:<resource>#g' \
    -e 's#arn:aws:lambda:[^[:space:]"]+#arn:aws:lambda:<region>:<aws-account-id>:<resource>#g' \
    -e 's/subnet-[A-Za-z0-9]+/<subnet-id>/g' \
    -e 's/sg-[A-Za-z0-9]+/<security-group-id>/g'
}

aws_cmd() {
  aws --region "$REGION" "$@"
}

verify_aws_access() {
  local identity_output
  if ! identity_output="$(aws_cmd sts get-caller-identity --output json 2>&1)"; then
    {
      echo "AWS credentials unavailable or expired for region=$REGION"
      echo "Refresh the AWS login/session, then rerun this check."
      echo "$identity_output"
    } | redact >&2
    exit 1
  fi

  echo "aws identity ok: account=$(jq -r '.Account' <<<"$identity_output")" | redact
}

positive_integer_arg() {
  local name="$1"
  local value="$2"
  if ! [[ "$value" =~ ^[1-9][0-9]*$ ]]; then
    echo "$name must be a positive integer; got $value" >&2
    exit 1
  fi
}

absolute_output_path() {
  local name="$1"
  local value="$2"
  case "$value" in
    /*) ;;
    *)
      echo "$name must be an absolute path; got $value" >&2
      exit 1
      ;;
  esac
}

task_env_value() {
  local name="$1"
  jq -r \
    --arg container "$CONTAINER_NAME" \
    --arg name "$name" \
    '.taskDefinition.containerDefinitions[]
      | select(.name == $container)
      | (.environment // [])[]?
      | select(.name == $name)
      | .value' "$task_json" \
    | head -n 1
}

list_latest_objects() {
  local bucket="$1"
  local prefix="$2"
  local limit="$3"
  aws_cmd s3api list-objects-v2 \
    --bucket "$bucket" \
    --prefix "$prefix" \
    --output json \
  | jq -c --argjson limit "$limit" '
      (.Contents // [])
      | sort_by(.LastModified, .Key)
      | reverse
      | .[0:$limit]
    '
}

latest_universe_snapshot_object_json() {
  local bucket="$1"
  local prefix="$2"
  aws_cmd s3api list-objects-v2 \
    --bucket "$bucket" \
    --prefix "$prefix" \
    --output json \
  | jq -c --arg prefix "$prefix" '
      def with_run_id_times:
        . as $object
        | ($object.Key | capture("run_id=l1_(?<start>[0-9]+)_(?<end>[0-9]+)_(?<generated>[0-9]+)")? // {}) as $run
        | $object + {
            run_start_ms:(($run.start // "0") | tonumber),
            run_end_ms:(($run.end // "0") | tonumber),
            run_generated_ms:(($run.generated // "0") | tonumber)
          };

      (.Contents // [])
      | map(with_run_id_times)
      | sort_by(.run_end_ms, .LastModified, .Key)
      | last as $last
      | if $last == null then
          {
            prefix:$prefix,
            selection:"latest_universe_as_of",
            lastModified:null,
            size:null,
            key:null,
            run_start_ms:null,
            run_end_ms:null,
            run_generated_ms:null
          }
        else
          {
            prefix:$prefix,
            selection:"latest_universe_as_of",
            lastModified:$last.LastModified,
            size:$last.Size,
            key:$last.Key,
            run_start_ms:$last.run_start_ms,
            run_end_ms:$last.run_end_ms,
            run_generated_ms:$last.run_generated_ms
          }
        end
    '
}

require_command aws
require_command jq
require_command sed
require_command mktemp
require_command date

positive_integer_arg "RESEARCH_BATCH_CANDIDATE_READ_LIMIT" "$CANDIDATE_READ_LIMIT"
positive_integer_arg "RESEARCH_BATCH_MAX_CANDIDATE_BUNDLE_COUNT" "$MAX_CANDIDATE_BUNDLE_COUNT"
positive_integer_arg "RESEARCH_BATCH_HISTORICAL_INDEX_READ_LIMIT" "$HISTORICAL_INDEX_READ_LIMIT"
positive_integer_arg "RESEARCH_BATCH_MAX_HISTORICAL_REPLAY_RUN_REF_COUNT" "$MAX_HISTORICAL_REPLAY_RUN_REF_COUNT"
positive_integer_arg "RESEARCH_BATCH_MAX_REPLAY_RUN_COUNT" "$MAX_REPLAY_RUN_COUNT"
case "$UNIVERSE_MODE" in
  current_approved | current_observed | legacy_retest) ;;
  *)
    echo "RESEARCH_BATCH_UNIVERSE_MODE must be current_approved, current_observed, or legacy_retest; got $UNIVERSE_MODE" >&2
    exit 1
    ;;
esac

if [[ -n "${RESEARCH_BATCH_MANIFEST_OUTPUT:-}" ]]; then
  MANIFEST_OUTPUT="$RESEARCH_BATCH_MANIFEST_OUTPUT"
else
  tmp_root="${TMPDIR:-/tmp}"
  tmp_root="${tmp_root%/}"
  MANIFEST_OUTPUT="$(mktemp "${tmp_root}/research-input-manifest.XXXXXX")"
fi
SUMMARY_OUTPUT="${RESEARCH_BATCH_SUMMARY_OUTPUT:-${MANIFEST_OUTPUT}.summary.json}"
absolute_output_path "RESEARCH_BATCH_MANIFEST_OUTPUT" "$MANIFEST_OUTPUT"
absolute_output_path "RESEARCH_BATCH_SUMMARY_OUTPUT" "$SUMMARY_OUTPUT"

echo "== ${APP_NAME} batch manifest builder =="
echo "region=$REGION"
echo "dispatcher=$DISPATCHER_FUNCTION"
echo "task_definition=$TASK_DEFINITION"
echo "candidate_read_limit=$CANDIDATE_READ_LIMIT"
echo "max_candidate_bundle_count=$MAX_CANDIDATE_BUNDLE_COUNT"
echo "historical_index_read_limit=$HISTORICAL_INDEX_READ_LIMIT"
echo "universe_mode=$UNIVERSE_MODE"
echo

verify_aws_access

lambda_json="$(mktemp)"
task_json="$(mktemp)"
candidate_p0_json="$(mktemp)"
candidate_p1_json="$(mktemp)"
candidate_p2_json="$(mktemp)"
candidate_objects_json="$(mktemp)"
candidate_records_json="$(mktemp)"
selected_candidates_json="$(mktemp)"
historical_index_objects_json="$(mktemp)"
universe_object_json="$(mktemp)"
universe_summary_json="$(mktemp)"
all_candidates_json="$(mktemp)"
trap 'rm -f "$lambda_json" "$task_json" "$candidate_p0_json" "$candidate_p1_json" "$candidate_p2_json" "$candidate_objects_json" "$candidate_records_json" "$selected_candidates_json" "$historical_index_objects_json" "$universe_object_json" "$universe_summary_json" "$all_candidates_json"' EXIT

aws_cmd lambda get-function-configuration \
  --function-name "$DISPATCHER_FUNCTION" \
  --output json > "$lambda_json"

aws_cmd ecs describe-task-definition \
  --task-definition "$TASK_DEFINITION" \
  --output json > "$task_json"

dispatch_mode="$(jq -r '.Environment.Variables.RESEARCH_DISPATCH_MODE // "run_task"' "$lambda_json")"
candidate_bucket="${RESEARCH_CANDIDATE_S3_BUCKET:-$(jq -r '.Environment.Variables.ALLOWED_SOURCE_BUCKET // ""' "$lambda_json")}"
if [[ -z "$candidate_bucket" || "$candidate_bucket" == "null" ]]; then
  candidate_bucket="$(jq -r '.Environment.Variables.ALLOWED_SOURCE_BUCKETS // ""' "$lambda_json" | tr ',' '\n' | grep 'intel-candidate' | head -n 1)"
fi
output_bucket="$(task_env_value RESEARCH_OUTPUT_S3_BUCKET)"
market_l1_bucket="$(task_env_value RESEARCH_MARKET_L1_S3_BUCKET)"

if [[ -z "$candidate_bucket" || "$candidate_bucket" == "null" ]]; then
  echo "candidate bucket is not discoverable; set RESEARCH_CANDIDATE_S3_BUCKET" >&2
  exit 1
fi
if [[ -z "$output_bucket" || "$output_bucket" == "null" ]]; then
  echo "RESEARCH_OUTPUT_S3_BUCKET is missing from task definition" >&2
  exit 1
fi
if [[ -z "$market_l1_bucket" || "$market_l1_bucket" == "null" ]]; then
  echo "RESEARCH_MARKET_L1_S3_BUCKET is missing from task definition" >&2
  exit 1
fi

{
  echo "dispatcher_mode=$dispatch_mode"
  echo "candidate_bucket=$candidate_bucket"
  echo "market_l1_bucket=$market_l1_bucket"
  echo "research_output_bucket=$output_bucket"
} | redact

latest_universe_snapshot_object_json \
  "$market_l1_bucket" \
  "symbol_universe_snapshot/run_id=" > "$universe_object_json"
universe_key="$(jq -r '.key // empty' "$universe_object_json")"
if [[ -n "$universe_key" ]]; then
  aws_cmd s3 cp "s3://${market_l1_bucket}/${universe_key}" - \
  | jq -c \
      --argjson object "$(cat "$universe_object_json")" '
        def members: ((.included_symbols // []) + (.excluded_symbols // []));
        def status_reason_counts:
          members
          | map(.status_reason // "unknown")
          | group_by(.)
          | map({reason:.[0], count:length})
          | sort_by(.count)
          | reverse;
        {
          present:true,
          key:$object.key,
          last_modified:$object.lastModified,
          selection:$object.selection,
          run_start_ms:$object.run_start_ms,
          run_end_ms:$object.run_end_ms,
          run_generated_ms:$object.run_generated_ms,
          schema_version,
          symbol_universe_snapshot_id,
          universe_as_of_ms,
          observed_symbols:[(.liquidity_rank_at_that_time // members)[]?.symbol_canonical],
          approved_symbols:[(.included_symbols // [])[]?.symbol_canonical],
          excluded_symbols:[(.excluded_symbols // [])[]?.symbol_canonical],
          observed_symbol_count:((.liquidity_rank_at_that_time // members) | length),
          approved_symbol_count:((.included_symbols // []) | length),
          excluded_symbol_count:((.excluded_symbols // []) | length),
          status_reason_counts:status_reason_counts
        }
      ' > "$universe_summary_json"
else
  jq -n -c '{
    present:false,
    observed_symbols:[],
    approved_symbols:[],
    excluded_symbols:[],
    observed_symbol_count:0,
    approved_symbol_count:0,
    excluded_symbol_count:0,
    status_reason_counts:[]
  }' > "$universe_summary_json"
fi

{
  jq -r '
    "latest_universe_selection=\(.selection // "absent")",
    "latest_universe_key=\(.key // "absent")",
    "latest_universe_observed_count=\(.observed_symbol_count)",
    "latest_universe_approved_count=\(.approved_symbol_count)"
  ' "$universe_summary_json"
} | redact

list_latest_objects "$candidate_bucket" "candidate-evidence-bundle/priority=p0/" "$CANDIDATE_READ_LIMIT" > "$candidate_p0_json"
list_latest_objects "$candidate_bucket" "candidate-evidence-bundle/priority=p1/" "$CANDIDATE_READ_LIMIT" > "$candidate_p1_json"
list_latest_objects "$candidate_bucket" "candidate-evidence-bundle/priority=p2/" "$CANDIDATE_READ_LIMIT" > "$candidate_p2_json"

jq -s -c \
  --argjson limit "$CANDIDATE_READ_LIMIT" \
  '[.[][]]
    | sort_by(.LastModified, .Key)
    | reverse
    | .[0:$limit]' \
  "$candidate_p0_json" "$candidate_p1_json" "$candidate_p2_json" > "$candidate_objects_json"

: > "$candidate_records_json"
while IFS= read -r object; do
  key="$(jq -r '.Key' <<<"$object")"
  last_modified="$(jq -r '.LastModified' <<<"$object")"
  size="$(jq -r '.Size' <<<"$object")"
  [[ -z "$key" || "$key" == "null" ]] && continue
  aws_cmd s3 cp "s3://${candidate_bucket}/${key}" - \
  | jq -c \
      --arg bucket "$candidate_bucket" \
      --arg key "$key" \
      --arg uri "s3://${candidate_bucket}/${key}" \
      --arg last_modified "$last_modified" \
      --argjson size "$size" '
        . as $record
        | if type == "array" then .[] else . end
        | select(type == "object")
        | {
            bucket:$bucket,
            key:$key,
            uri:$uri,
            last_modified:$last_modified,
            size:$size,
            candidate_id:(.candidate_id // null),
            candidate_lifecycle_key:(.candidate_lifecycle_key // null),
            candidate_class:(.candidate_class // null),
            research_priority:(.research_priority // null),
            research_eligible:(.research_eligible // false),
            symbols:(
              (.normalized_symbols // .symbols // [])
              | if type == "array" then
                  map(if type == "string" then . else (.symbol_canonical // .symbol // .asset // empty) end)
                elif type == "string" then [.]
                else []
                end
            ),
            allowed_horizons:(.allowed_horizons // []),
            approved_universe_symbol:(.approved_universe_symbol // false),
            forbidden_lookahead_boundary_ms:(.forbidden_lookahead_boundary_ms // null),
            universe_as_of_ms:(.universe_as_of_ms // null),
            symbol_universe_snapshot_id:(.symbol_universe_snapshot_id // null)
          }
      ' >> "$candidate_records_json"
done < <(jq -c '.[]' "$candidate_objects_json")

jq -s -c \
  --arg mode "$UNIVERSE_MODE" \
  --argjson universe "$(cat "$universe_summary_json")" '
    def all_symbols_in($allowed):
      (.symbols | length) > 0
      and all(.symbols[]; . as $symbol | ($allowed | index($symbol)));
    def horizon_ms($h):
      if $h == "15m" then 900000
      elif $h == "1h" then 3600000
      elif $h == "4h" then 14400000
      elif $h == "24h" then 86400000
      elif $h == "72h" then 259200000
      elif $h == "7d" then 604800000
      else null
      end;
    def absolute_max_horizon_ms: 259200000;
    def horizon_contract_valid:
      (.allowed_horizons // []) as $horizons
      | ($horizons | length) > 0
        and all($horizons[]; (horizon_ms(.) != null and horizon_ms(.) <= absolute_max_horizon_ms));
    def horizon_contract_reasons:
      (.allowed_horizons // []) as $horizons
      | (
          if ($horizons | length) == 0
          then ["missing_allowed_horizons"]
          else []
          end
        )
        + [
          $horizons[]
          | select(horizon_ms(.) == null)
          | "unsupported_horizon:" + .
        ]
        + [
          $horizons[]
          | select((horizon_ms(.) // 0) > absolute_max_horizon_ms)
          | "holding_horizon_contract_violation:" + .
        ];

    map(select(.candidate_id != null and .research_eligible == true))
    | map(. + {
        current_universe_snapshot_id:($universe.symbol_universe_snapshot_id // null),
        current_universe_as_of_ms:($universe.universe_as_of_ms // null),
        current_universe_observed:all_symbols_in($universe.observed_symbols // []),
        current_universe_approved:all_symbols_in($universe.approved_symbols // []),
        bundle_current_universe_match:(.symbol_universe_snapshot_id == ($universe.symbol_universe_snapshot_id // null)),
        universe_selection_mode:$mode,
        batch_horizon_contract_valid:horizon_contract_valid,
        batch_horizon_contract_reasons:horizon_contract_reasons
      })
  ' "$candidate_records_json" > "$all_candidates_json"

jq -c \
  --arg mode "$UNIVERSE_MODE" \
  --argjson max "$MAX_CANDIDATE_BUNDLE_COUNT" '
    map(select(
      .batch_horizon_contract_valid == true
      and if $mode == "current_approved" then .current_universe_approved == true
      elif $mode == "current_observed" then .current_universe_observed == true
      elif $mode == "legacy_retest" then .research_eligible == true
      else false
      end
    ))
    | sort_by(.last_modified, .key)
    | reverse
    | reduce .[] as $candidate ({};
        if has($candidate.candidate_id) then .
        else .[$candidate.candidate_id] = $candidate
        end
      )
    | [.[]]
    | sort_by(.last_modified, .key)
    | reverse
    | .[0:$max]
  ' "$all_candidates_json" > "$selected_candidates_json"

selected_candidate_count="$(jq 'length' "$selected_candidates_json")"
if [[ "$selected_candidate_count" == "0" ]]; then
  jq -n \
    --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --arg manifest_output "$MANIFEST_OUTPUT" \
    --arg summary_output "$SUMMARY_OUTPUT" \
    --arg region "$REGION" \
    --arg dispatch_mode "$dispatch_mode" \
    --arg universe_mode "$UNIVERSE_MODE" \
    --argjson candidate_read_limit "$CANDIDATE_READ_LIMIT" \
    --argjson max_candidate_bundle_count "$MAX_CANDIDATE_BUNDLE_COUNT" \
    --argjson universe "$(cat "$universe_summary_json")" \
    --argjson candidates "$(cat "$all_candidates_json")" \
    '{
      generated_at:$generated_at,
      manifest_output:$manifest_output,
      summary_output:$summary_output,
      region:$region,
      dispatch_mode:$dispatch_mode,
      universe_mode:$universe_mode,
      safety:{
        s3_write:false,
        ecs_task_started:false,
        dispatcher_mode_changed:false,
        local_manifest_only:true,
        selected_candidates_require_current_universe:($universe_mode != "legacy_retest")
      },
      latest_universe:$universe,
      candidate_read_limit:$candidate_read_limit,
      max_candidate_bundle_count:$max_candidate_bundle_count,
      selected_candidate_count:0,
      scanned_research_eligible_candidate_count:($candidates | length),
      current_observed_candidate_count:([$candidates[] | select(.current_universe_observed == true)] | length),
      current_approved_candidate_count:([$candidates[] | select(.current_universe_approved == true)] | length),
      legacy_bundle_approved_candidate_count:([$candidates[] | select(.approved_universe_symbol == true)] | length),
      horizon_contract_valid_candidate_count:([$candidates[] | select(.batch_horizon_contract_valid == true)] | length),
      horizon_contract_invalid_candidate_count:([$candidates[] | select(.batch_horizon_contract_valid != true)] | length),
      excluded_horizon_contract_violations:(
        [$candidates[] | select(.batch_horizon_contract_valid != true)]
        | map({
            candidate_id,
            symbols,
            allowed_horizons,
            reasons:.batch_horizon_contract_reasons,
            last_modified,
            key
          })
        | .[0:20]
      ),
      blocked_reason:"no_candidates_match_universe_mode_or_horizon_contract"
    }' > "$SUMMARY_OUTPUT"
  jq -n \
    --arg research_packet_id "$RESEARCH_PACKET_ID" \
    --arg run_scope "$RUN_SCOPE" \
    --argjson max_candidates "$MAX_CANDIDATE_BUNDLE_COUNT" \
    --argjson max_history "$MAX_HISTORICAL_REPLAY_RUN_REF_COUNT" \
    --argjson max_replay "$MAX_REPLAY_RUN_COUNT" \
    '{
      schema_version:"research_input_manifest_v1",
      research_packet_id:$research_packet_id,
      run_scope:$run_scope,
      candidate_bundle_refs:[],
      historical_replay_run_index_refs:[],
      runtime_budget_policy:{
        max_candidate_bundle_count:$max_candidates,
        max_market_artifact_ref_count:2000,
        max_shadow_validation_run_ref_count:10000,
        max_hypothesis_harness_result_ref_count:10000,
        max_oss_adapter_run_ref_count:10000,
        max_historical_replay_run_ref_count:$max_history,
        max_replay_run_count:$max_replay
      }
    }' > "$MANIFEST_OUTPUT"
  {
    echo "manifest_output=$MANIFEST_OUTPUT"
    echo "summary_output=$SUMMARY_OUTPUT"
    echo "selected_candidate_count=0"
    jq -r '
      "scanned_research_eligible_candidate_count=\(.scanned_research_eligible_candidate_count)",
      "current_observed_candidate_count=\(.current_observed_candidate_count)",
      "current_approved_candidate_count=\(.current_approved_candidate_count)",
      "legacy_bundle_approved_candidate_count=\(.legacy_bundle_approved_candidate_count)",
      "horizon_contract_invalid_candidate_count=\(.horizon_contract_invalid_candidate_count)",
      "blocked_reason=\(.blocked_reason)"
    ' "$SUMMARY_OUTPUT"
  } | redact
  echo "no candidate bundles matched RESEARCH_BATCH_UNIVERSE_MODE=$UNIVERSE_MODE" >&2
  exit 1
fi

list_latest_objects "$output_bucket" "replay-run-index/" "$HISTORICAL_INDEX_READ_LIMIT" \
| jq -c --arg bucket "$output_bucket" '
    map({
      bucket:$bucket,
      key:.Key,
      uri:("s3://" + $bucket + "/" + .Key),
      last_modified:.LastModified,
      size:.Size
    })
  ' > "$historical_index_objects_json"

jq -n \
  --arg research_packet_id "$RESEARCH_PACKET_ID" \
  --arg run_scope "$RUN_SCOPE" \
  --argjson candidates "$(cat "$selected_candidates_json")" \
  --argjson indexes "$(cat "$historical_index_objects_json")" \
  --argjson max_candidates "$MAX_CANDIDATE_BUNDLE_COUNT" \
  --argjson max_history "$MAX_HISTORICAL_REPLAY_RUN_REF_COUNT" \
  --argjson max_replay "$MAX_REPLAY_RUN_COUNT" \
  '{
    schema_version:"research_input_manifest_v1",
    research_packet_id:$research_packet_id,
    run_scope:$run_scope,
    candidate_bundle_refs:($candidates | map({uri:.uri})),
    historical_replay_run_index_refs:($indexes | map({uri:.uri})),
    runtime_budget_policy:{
      max_candidate_bundle_count:$max_candidates,
      max_market_artifact_ref_count:2000,
      max_shadow_validation_run_ref_count:10000,
      max_hypothesis_harness_result_ref_count:10000,
      max_oss_adapter_run_ref_count:10000,
      max_historical_replay_run_ref_count:$max_history,
      max_replay_run_count:$max_replay
    }
  }' > "$MANIFEST_OUTPUT"

jq -n \
  --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --arg manifest_output "$MANIFEST_OUTPUT" \
  --arg summary_output "$SUMMARY_OUTPUT" \
  --arg region "$REGION" \
  --arg dispatch_mode "$dispatch_mode" \
  --arg universe_mode "$UNIVERSE_MODE" \
  --arg run_scope "$RUN_SCOPE" \
  --arg research_packet_id "$RESEARCH_PACKET_ID" \
  --argjson candidate_read_limit "$CANDIDATE_READ_LIMIT" \
  --argjson max_candidate_bundle_count "$MAX_CANDIDATE_BUNDLE_COUNT" \
  --argjson universe "$(cat "$universe_summary_json")" \
  --argjson scanned_candidates "$(cat "$all_candidates_json")" \
  --argjson candidates "$(cat "$selected_candidates_json")" \
  --argjson indexes "$(cat "$historical_index_objects_json")" \
  '{
    generated_at:$generated_at,
    manifest_output:$manifest_output,
    summary_output:$summary_output,
    region:$region,
    dispatch_mode:$dispatch_mode,
    universe_mode:$universe_mode,
    safety:{
      s3_write:false,
      ecs_task_started:false,
      dispatcher_mode_changed:false,
      local_manifest_only:true,
      selected_candidates_require_current_universe:($universe_mode != "legacy_retest")
    },
    latest_universe:$universe,
    candidate_read_limit:$candidate_read_limit,
    max_candidate_bundle_count:$max_candidate_bundle_count,
    research_packet_id:$research_packet_id,
    run_scope:$run_scope,
    scanned_research_eligible_candidate_count:($scanned_candidates | length),
    selected_candidate_count:($candidates | length),
    distinct_candidate_symbols:([$candidates[].symbols[]?] | unique | sort),
    candidate_class_counts:(
      [$candidates[].candidate_class]
      | group_by(.)
      | map({candidate_class:.[0], count:length})
    ),
    research_priority_counts:(
      [$candidates[].research_priority]
      | group_by(.)
      | map({research_priority:.[0], count:length})
    ),
    allowed_horizons:([$candidates[].allowed_horizons[]?] | unique | sort),
    approved_bundle_candidate_count:([$candidates[] | select(.approved_universe_symbol == true)] | length),
    current_observed_candidate_count:([$scanned_candidates[] | select(.current_universe_observed == true)] | length),
    current_approved_candidate_count:([$scanned_candidates[] | select(.current_universe_approved == true)] | length),
    horizon_contract_valid_candidate_count:([$scanned_candidates[] | select(.batch_horizon_contract_valid == true)] | length),
    horizon_contract_invalid_candidate_count:([$scanned_candidates[] | select(.batch_horizon_contract_valid != true)] | length),
    excluded_horizon_contract_violations:(
      [$scanned_candidates[] | select(.batch_horizon_contract_valid != true)]
      | map({
          candidate_id,
          symbols,
          allowed_horizons,
          reasons:.batch_horizon_contract_reasons,
          last_modified,
          key
        })
      | .[0:20]
    ),
    selected_current_observed_candidate_count:([$candidates[] | select(.current_universe_observed == true)] | length),
    selected_current_approved_candidate_count:([$candidates[] | select(.current_universe_approved == true)] | length),
    selected_horizon_contract_valid_count:([$candidates[] | select(.batch_horizon_contract_valid == true)] | length),
    selected_bundle_current_universe_match_count:([$candidates[] | select(.bundle_current_universe_match == true)] | length),
    historical_replay_run_index_ref_count:($indexes | length),
    selected_candidates:($candidates | map({
      candidate_id,
      candidate_class,
      research_priority,
      symbols,
      allowed_horizons,
      approved_universe_symbol,
      current_universe_observed,
      current_universe_approved,
      bundle_current_universe_match,
      batch_horizon_contract_valid,
      last_modified,
      key
    }))
  }' > "$SUMMARY_OUTPUT"

{
  echo "manifest_output=$MANIFEST_OUTPUT"
  echo "summary_output=$SUMMARY_OUTPUT"
  jq -r '
    "selected_candidate_count=\(.selected_candidate_count)",
    "distinct_candidate_symbols=\(.distinct_candidate_symbols | join(","))",
    "allowed_horizons=\(.allowed_horizons | join(","))",
    "universe_mode=\(.universe_mode)",
    "current_observed_candidate_count=\(.current_observed_candidate_count)",
    "current_approved_candidate_count=\(.current_approved_candidate_count)",
    "horizon_contract_invalid_candidate_count=\(.horizon_contract_invalid_candidate_count)",
    "selected_current_approved_candidate_count=\(.selected_current_approved_candidate_count)",
    "selected_horizon_contract_valid_count=\(.selected_horizon_contract_valid_count)",
    "historical_replay_run_index_ref_count=\(.historical_replay_run_index_ref_count)",
    "safety=s3_write:\(.safety.s3_write),ecs_task_started:\(.safety.ecs_task_started),dispatcher_mode_changed:\(.safety.dispatcher_mode_changed)"
  ' "$SUMMARY_OUTPUT"
  echo
  echo "local validation command:"
  printf 'AWS_PROFILE=<sso-profile> AWS_REGION=%q cargo run -- --input-manifest-file %q --market-l1-s3-bucket %q --output-dir /absolute/path/to/local-research-output\n' \
    "$REGION" "$MANIFEST_OUTPUT" "$market_l1_bucket"
} | redact

echo "research batch manifest build completed"
