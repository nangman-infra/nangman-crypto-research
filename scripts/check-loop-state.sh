#!/usr/bin/env bash
set -euo pipefail

REGION="${AWS_REGION:-${AWS_DEFAULT_REGION:-ap-northeast-2}}"
DISPATCHER_FUNCTION="${RESEARCH_DISPATCHER_FUNCTION:-lmbd-nangman-dev-research-apn2}"
TASK_DEFINITION="${RESEARCH_ECS_TASK_DEFINITION:-td-nangman-dev-research-apn2}"
CONTAINER_NAME="${RESEARCH_ECS_CONTAINER:-research-app}"
CANDIDATE_READ_LIMIT="${RESEARCH_LOOP_STATE_CANDIDATE_READ_LIMIT:-50}"
EXPECTED_MAJOR_UNIVERSE_SIZE="${RESEARCH_EXPECTED_MAJOR_UNIVERSE_SIZE:-50}"

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

latest_object_json() {
  local bucket="$1"
  local prefix="$2"
  aws_cmd s3api list-objects-v2 \
    --bucket "$bucket" \
    --prefix "$prefix" \
    --output json \
  | jq -c --arg prefix "$prefix" '
      (.Contents // []) | sort_by(.LastModified, .Key) | last as $last
      | if $last == null then
          {prefix:$prefix,lastModified:null,size:null,key:null}
        else
          {prefix:$prefix,lastModified:$last.LastModified,size:$last.Size,key:$last.Key}
        end
    '
}

require_command aws
require_command jq
require_command sed
require_command mktemp

echo "== research loop state =="
echo "region=$REGION"
echo "dispatcher=$DISPATCHER_FUNCTION"
echo "task_definition=$TASK_DEFINITION"
echo "candidate_read_limit=$CANDIDATE_READ_LIMIT"
echo

verify_aws_access

lambda_json="$(mktemp)"
task_json="$(mktemp)"
candidate_p0_json="$(mktemp)"
candidate_p1_json="$(mktemp)"
candidate_p2_json="$(mktemp)"
candidate_objects_json="$(mktemp)"
candidate_records_json="$(mktemp)"
trap 'rm -f "$lambda_json" "$task_json" "$candidate_p0_json" "$candidate_p1_json" "$candidate_p2_json" "$candidate_objects_json" "$candidate_records_json"' EXIT

aws_cmd lambda get-function-configuration \
  --function-name "$DISPATCHER_FUNCTION" \
  --output json > "$lambda_json"

aws_cmd ecs describe-task-definition \
  --task-definition "$TASK_DEFINITION" \
  --output json > "$task_json"

lambda_state="$(jq -r '.State' "$lambda_json")"
lambda_update_status="$(jq -r '.LastUpdateStatus' "$lambda_json")"
dispatch_mode="$(jq -r '.Environment.Variables.RESEARCH_DISPATCH_MODE // "run_task"' "$lambda_json")"
candidate_bucket="${RESEARCH_CANDIDATE_S3_BUCKET:-$(jq -r '.Environment.Variables.ALLOWED_SOURCE_BUCKET // ""' "$lambda_json")}"
if [[ -z "$candidate_bucket" || "$candidate_bucket" == "null" ]]; then
  candidate_bucket="$(jq -r '.Environment.Variables.ALLOWED_SOURCE_BUCKETS // ""' "$lambda_json" | tr ',' '\n' | grep 'intel-candidate' | head -n 1)"
fi

task_revision="$(jq -r '.taskDefinition.revision' "$task_json")"
task_status="$(jq -r '.taskDefinition.status' "$task_json")"
cpu_arch="$(jq -r '.taskDefinition.runtimePlatform.cpuArchitecture' "$task_json")"
os_family="$(jq -r '.taskDefinition.runtimePlatform.operatingSystemFamily' "$task_json")"
readonly_root="$(jq -r --arg name "$CONTAINER_NAME" '.taskDefinition.containerDefinitions[] | select(.name == $name) | .readonlyRootFilesystem' "$task_json")"
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

runtime_summary="$(jq -n -c \
  --arg lambda_state "$lambda_state" \
  --arg lambda_update_status "$lambda_update_status" \
  --arg dispatch_mode "$dispatch_mode" \
  --arg task_definition "${TASK_DEFINITION}:${task_revision}" \
  --arg task_status "$task_status" \
  --arg cpu_arch "$cpu_arch" \
  --arg os_family "$os_family" \
  --arg readonly_root "$readonly_root" \
  '{
    dispatcher_lambda_state:$lambda_state,
    dispatcher_update_status:$lambda_update_status,
    dispatcher_mode:$dispatch_mode,
    task_definition:$task_definition,
    task_status:$task_status,
    cpu_architecture:$cpu_arch,
    operating_system_family:$os_family,
    readonly_root_filesystem:($readonly_root == "true"),
    runtime_alive:(
      $lambda_state == "Active"
      and $lambda_update_status == "Successful"
      and $task_status == "ACTIVE"
      and $cpu_arch == "ARM64"
      and $os_family == "LINUX"
      and $readonly_root == "true"
    )
  }')"

universe_object="$(latest_object_json "$market_l1_bucket" "symbol_universe_snapshot/run_id=")"
universe_key="$(jq -r '.key // empty' <<<"$universe_object")"
if [[ -n "$universe_key" ]]; then
  universe_summary="$(
    aws_cmd s3 cp "s3://${market_l1_bucket}/${universe_key}" - \
    | jq -c \
      --argjson expected "$EXPECTED_MAJOR_UNIVERSE_SIZE" \
      --arg key "$universe_key" \
      --arg last_modified "$(jq -r '.lastModified' <<<"$universe_object")" '
        def members: ((.included_symbols // []) + (.excluded_symbols // []));
        def top_reasons:
          [(.excluded_symbols // [])[]?.status_reason]
          | group_by(.)
          | map({reason:.[0], count:length})
          | sort_by(.count)
          | reverse
          | .[0:5];
        {
          present:true,
          key:$key,
          last_modified:$last_modified,
          schema_version,
          symbol_universe_snapshot_id,
          universe_as_of_ms,
          expected_major_universe_size:$expected,
          observed_symbol_count:((.liquidity_rank_at_that_time // members) | length),
          approved_symbol_count:((.included_symbols // []) | length),
          excluded_symbol_count:((.excluded_symbols // []) | length),
          major_coverage_complete:(((.liquidity_rank_at_that_time // members) | length) >= $expected),
          approved_major_coverage_complete:(((.included_symbols // []) | length) >= $expected),
          top_observed_symbols:[(.liquidity_rank_at_that_time // [])[]?.symbol_canonical][0:$expected],
          approved_symbols:[(.included_symbols // [])[]?.symbol_canonical][0:$expected],
          top_exclusion_reasons:top_reasons
        }
      '
  )"
else
  universe_summary="$(jq -n -c --argjson expected "$EXPECTED_MAJOR_UNIVERSE_SIZE" '{
    present:false,
    expected_major_universe_size:$expected,
    observed_symbol_count:0,
    approved_symbol_count:0,
    excluded_symbol_count:0,
    major_coverage_complete:false,
    approved_major_coverage_complete:false,
    top_observed_symbols:[],
    approved_symbols:[],
    top_exclusion_reasons:[]
  }')"
fi

aws_cmd s3api list-objects-v2 \
  --bucket "$candidate_bucket" \
  --prefix "candidate-evidence-bundle/priority=p0/" \
  --output json > "$candidate_p0_json"
aws_cmd s3api list-objects-v2 \
  --bucket "$candidate_bucket" \
  --prefix "candidate-evidence-bundle/priority=p1/" \
  --output json > "$candidate_p1_json"
aws_cmd s3api list-objects-v2 \
  --bucket "$candidate_bucket" \
  --prefix "candidate-evidence-bundle/priority=p2/" \
  --output json > "$candidate_p2_json"

jq -s -c \
  --argjson limit "$CANDIDATE_READ_LIMIT" \
  '[.[].Contents[]?]
    | sort_by(.LastModified, .Key)
    | reverse
    | .[0:$limit]' \
  "$candidate_p0_json" "$candidate_p1_json" "$candidate_p2_json" > "$candidate_objects_json"

: > "$candidate_records_json"
while IFS= read -r key; do
  [[ -z "$key" ]] && continue
  aws_cmd s3 cp "s3://${candidate_bucket}/${key}" - \
  | jq -c --arg key "$key" '
      def symbol_names:
        (.normalized_symbols // .symbols // [])
        | if type == "array" then
            map(
              if type == "string" then .
              else (.symbol_canonical // .symbol // .asset // empty)
              end
            )
          elif type == "string" then [.]
          else []
          end;
      . as $record
      | if type == "array" then .[] else . end
      | select(type == "object")
      | {
          source_key:$key,
          candidate_id:(.candidate_id // null),
          candidate_class:(.candidate_class // null),
          research_priority:(.research_priority // null),
          symbols:symbol_names,
          allowed_horizons:(.allowed_horizons // []),
          approved_universe_symbol:(.approved_universe_symbol // null),
          symbol_universe_snapshot_id:(.symbol_universe_snapshot_id // null)
        }
    ' >> "$candidate_records_json"
done < <(jq -r '.[].Key' "$candidate_objects_json")

candidate_summary="$(jq -s -c \
  --argjson object_count "$(jq 'length' "$candidate_objects_json")" \
  '{
    recent_bundle_object_count:$object_count,
    recent_candidate_record_count:length,
    distinct_candidate_symbols:([.[].symbols[]?] | unique | sort),
    distinct_candidate_symbol_count:([.[].symbols[]?] | unique | length),
    latest_candidates:[.[0:10][] | {
      candidate_id,
      candidate_class,
      research_priority,
      symbols,
      allowed_horizons,
      approved_universe_symbol
    }]
  }' "$candidate_records_json")"

report_object="$(latest_object_json "$output_bucket" "research-run-report/")"
replay_object="$(latest_object_json "$output_bucket" "replay-run/")"
index_object="$(latest_object_json "$output_bucket" "replay-run-index/")"
shadow_object="$(latest_object_json "$output_bucket" "shadow-validation-run/")"
paper_object="$(latest_object_json "$output_bucket" "paper-trade-run/")"

report_key="$(jq -r '.key // empty' <<<"$report_object")"
if [[ -n "$report_key" ]]; then
  report_summary="$(
    aws_cmd s3 cp "s3://${output_bucket}/${report_key}" - \
    | jq -c \
      --arg key "$report_key" \
      --arg last_modified "$(jq -r '.lastModified' <<<"$report_object")" '
        def bias_counts:
          (.summary_findings // [])
          | group_by(.bias)
          | map({bias:.[0].bias, count:length})
          | sort_by(.bias);
        {
          present:true,
          key:$key,
          last_modified:$last_modified,
          schema_version,
          research_run_status,
          source_candidate_count:((.source_candidate_ids // []) | length),
          replay_run_count:((.replay_run_ids // []) | length),
          partition_count:(.partition_count // ((.partition_aggregates // []) | length)),
          top_symbols:(.top_symbols // []),
          surviving_candidate_count:((.surviving_candidate_keys // []) | length),
          retest_candidate_count:((.retest_candidate_keys // []) | length),
          pruned_candidate_count:((.pruned_candidate_keys // []) | length),
          shadow_validation_count:((.shadow_validation_runs // []) | length),
          paper_trade_candidate_count:((.paper_trade_candidates // []) | length),
          bias_counts:bias_counts,
          promotion_bias_count:([
            (.summary_findings // [])[]?
            | select((.bias // "") | startswith("PROMOTE_TO_"))
          ] | length)
        }
      '
  )"
else
  report_summary="$(jq -n -c '{
    present:false,
    source_candidate_count:0,
    replay_run_count:0,
    partition_count:0,
    top_symbols:[],
    surviving_candidate_count:0,
    retest_candidate_count:0,
    pruned_candidate_count:0,
    shadow_validation_count:0,
    paper_trade_candidate_count:0,
    bias_counts:[],
    promotion_bias_count:0
  }')"
fi

prefix_summary="$(jq -n -c \
  --argjson report "$report_object" \
  --argjson replay "$replay_object" \
  --argjson index "$index_object" \
  --argjson shadow "$shadow_object" \
  --argjson paper "$paper_object" \
  '{
    research_run_report:$report,
    replay_run:$replay,
    replay_run_index:$index,
    shadow_validation_run:$shadow,
    paper_trade_run:$paper
  }')"

jq -n \
  --arg region "$REGION" \
  --arg candidate_bucket "$candidate_bucket" \
  --arg market_l1_bucket "$market_l1_bucket" \
  --arg output_bucket "$output_bucket" \
  --argjson runtime "$runtime_summary" \
  --argjson universe "$universe_summary" \
  --argjson candidates "$candidate_summary" \
  --argjson report "$report_summary" \
  --argjson prefixes "$prefix_summary" \
  '{
    region:$region,
    buckets:{
      candidate:$candidate_bucket,
      market_l1:$market_l1_bucket,
      research_output:$output_bucket
    },
    stage_state:{
      runtime_alive:$runtime.runtime_alive,
      dispatcher_auto_research_enabled:($runtime.dispatcher_mode == "run_task"),
      major50_universe_observed:$universe.major_coverage_complete,
      major50_universe_approved:$universe.approved_major_coverage_complete,
      candidate_generated:($candidates.recent_candidate_record_count > 0),
      artifact_created:($prefixes.research_run_report.key != null and $prefixes.replay_run.key != null and $prefixes.replay_run_index.key != null),
      research_replay_completed:($report.present and $report.replay_run_count > 0),
      promotion_passed:($report.promotion_bias_count > 0 or $report.shadow_validation_count > 0 or $report.paper_trade_candidate_count > 0),
      shadow_created:($prefixes.shadow_validation_run.key != null),
      paper_created:($prefixes.paper_trade_run.key != null),
      live_enabled:false
    },
    major50_universe:$universe,
    recent_candidates:$candidates,
    latest_research_report:$report,
    latest_prefixes:$prefixes,
    bottlenecks:([
      if ($runtime.dispatcher_mode != "run_task") then "dispatcher_not_run_task" else empty end,
      if ($universe.major_coverage_complete | not) then "major50_observed_universe_incomplete" else empty end,
      if ($universe.approved_major_coverage_complete | not) then "major50_approved_universe_incomplete" else empty end,
      if ($candidates.recent_candidate_record_count == 0) then "no_recent_candidate_bundles" else empty end,
      if (($report.present | not) or $report.replay_run_count == 0) then "research_replay_not_completed" else empty end,
      if ($report.promotion_bias_count == 0 and $report.shadow_validation_count == 0) then "no_promoted_shadow_candidate" else empty end,
      if ($prefixes.shadow_validation_run.key == null) then "shadow_output_absent" else empty end,
      if ($prefixes.paper_trade_run.key == null) then "paper_output_absent" else empty end
    ])
  }' | redact

echo "research loop state check completed"
