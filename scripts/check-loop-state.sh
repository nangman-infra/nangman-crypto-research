#!/usr/bin/env bash
set -euo pipefail

REGION="${AWS_REGION:-${AWS_DEFAULT_REGION:-ap-northeast-2}}"
DISPATCHER_FUNCTION="${RESEARCH_DISPATCHER_FUNCTION:-lmbd-nangman-dev-research-apn2}"
TASK_DEFINITION="${RESEARCH_ECS_TASK_DEFINITION:-td-nangman-dev-research-apn2}"
CONTAINER_NAME="${RESEARCH_ECS_CONTAINER:-research-app}"
CANDIDATE_READ_LIMIT="${RESEARCH_LOOP_STATE_CANDIDATE_READ_LIMIT:-1000}"
REPORT_READ_LIMIT="${RESEARCH_LOOP_STATE_REPORT_READ_LIMIT:-100}"
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

echo "== research loop state =="
echo "region=$REGION"
echo "dispatcher=$DISPATCHER_FUNCTION"
echo "task_definition=$TASK_DEFINITION"
echo "candidate_read_limit=$CANDIDATE_READ_LIMIT"
echo "report_read_limit=$REPORT_READ_LIMIT"
echo

verify_aws_access

lambda_json="$(mktemp)"
task_json="$(mktemp)"
candidate_p0_json="$(mktemp)"
candidate_p1_json="$(mktemp)"
candidate_p2_json="$(mktemp)"
candidate_objects_json="$(mktemp)"
candidate_records_json="$(mktemp)"
report_objects_json="$(mktemp)"
report_records_json="$(mktemp)"
trap 'rm -f "$lambda_json" "$task_json" "$candidate_p0_json" "$candidate_p1_json" "$candidate_p2_json" "$candidate_objects_json" "$candidate_records_json" "$report_objects_json" "$report_records_json"' EXIT

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

universe_object="$(latest_universe_snapshot_object_json "$market_l1_bucket" "symbol_universe_snapshot/run_id=")"
universe_key="$(jq -r '.key // empty' <<<"$universe_object")"
if [[ -n "$universe_key" ]]; then
  universe_summary="$(
    aws_cmd s3 cp "s3://${market_l1_bucket}/${universe_key}" - \
    | jq -c \
      --argjson expected "$EXPECTED_MAJOR_UNIVERSE_SIZE" \
      --argjson object "$universe_object" '
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
          key:$object.key,
          last_modified:$object.lastModified,
          selection:$object.selection,
          run_start_ms:$object.run_start_ms,
          run_end_ms:$object.run_end_ms,
          run_generated_ms:$object.run_generated_ms,
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
  --argjson read_limit "$CANDIDATE_READ_LIMIT" \
  --argjson object_count "$(jq 'length' "$candidate_objects_json")" \
  '{
    candidate_read_limit:$read_limit,
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
          research_packet_id,
          run_scope,
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
          gate_biases:([(.partition_aggregates // [])[].gate_bias] | unique | sort),
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
    gate_biases:[],
    promotion_bias_count:0
  }')"
fi

aws_cmd s3api list-objects-v2 \
  --bucket "$output_bucket" \
  --prefix "research-run-report/" \
  --output json \
| jq -c --argjson limit "$REPORT_READ_LIMIT" '
    (.Contents // [])
    | sort_by(.LastModified, .Key)
    | reverse
    | .[0:$limit]' > "$report_objects_json"

: > "$report_records_json"
while IFS= read -r object_json; do
  key="$(jq -r '.Key' <<<"$object_json")"
  last_modified="$(jq -r '.LastModified' <<<"$object_json")"
  [[ -z "$key" || "$key" == "null" ]] && continue
  aws_cmd s3 cp "s3://${output_bucket}/${key}" - \
  | jq -c \
      --arg key "$key" \
      --arg last_modified "$last_modified" '
        {
          key:$key,
          last_modified:$last_modified,
          schema_version,
          research_packet_id,
          run_scope,
          research_run_status,
          source_candidate_count:((.source_candidate_ids // []) | length),
          replay_run_count:((.replay_run_ids // []) | length),
          partition_count:(.partition_count // ((.partition_aggregates // []) | length)),
          top_symbols:(.top_symbols // []),
          partition_symbols:([(.partition_aggregates // [])[].symbol_canonical] | unique | sort),
          gate_biases:([(.partition_aggregates // [])[].gate_bias] | unique | sort),
          shadow_validation_count:((.shadow_validation_runs // []) | length),
          paper_trade_candidate_count:((.paper_trade_candidates // []) | length),
          promotion_bias_count:([
            (.summary_findings // [])[]?
            | select((.bias // "") | startswith("PROMOTE_TO_"))
          ] | length)
        }
      ' >> "$report_records_json"
done < <(jq -c '.[]' "$report_objects_json")

current_approved_shard_batch_summary="$(jq -s -c '
  def shard_meta:
    (.research_packet_id // "")
    | capture("^(?<dispatch_group_id>.*)_shard(?<shard_number>[0-9]+)of(?<shard_count>[0-9]+)$")?;

  def empty_batch:
    {
      present:false,
      selection:"largest_complete_current_approved_shard_batch",
      dispatch_group_id:null,
      report_count:0,
      expected_shard_count:0,
      complete:false,
      first_last_modified:null,
      last_modified:null,
      source_candidate_count:0,
      replay_run_count:0,
      top_symbols:[],
      gate_biases:[],
      statuses:[],
      promotion_bias_count:0,
      shadow_validation_count:0,
      paper_trade_candidate_count:0
    };

  map(
    select(.run_scope == "current_approved_auto_research_validation_shard")
    | . + {shard_meta:shard_meta}
    | select(.shard_meta != null)
  )
  | group_by(.shard_meta.dispatch_group_id)
  | map(
      . as $reports
      | ($reports | map((.shard_meta.shard_count // "0") | tonumber) | max) as $expected_shard_count
      | ($reports | map((.shard_meta.shard_number // "0") | tonumber) | unique | sort) as $shard_numbers
      | {
          present:true,
          selection:"largest_complete_current_approved_shard_batch",
          dispatch_group_id:($reports[0].shard_meta.dispatch_group_id),
          report_count:($reports | length),
          expected_shard_count:$expected_shard_count,
          complete:(($shard_numbers | length) == $expected_shard_count),
          shard_numbers:$shard_numbers,
          first_last_modified:($reports | map(.last_modified) | min),
          last_modified:($reports | map(.last_modified) | max),
          source_candidate_count:($reports | map(.source_candidate_count) | add // 0),
          replay_run_count:($reports | map(.replay_run_count) | add // 0),
          top_symbols:($reports | map((.partition_symbols // [])[]?, (.top_symbols // [])[]?) | unique | sort),
          gate_biases:($reports | map((.gate_biases // [])[]?) | unique | sort),
          statuses:($reports | map(.research_run_status) | unique | sort),
          promotion_bias_count:($reports | map(.promotion_bias_count) | add // 0),
          shadow_validation_count:($reports | map(.shadow_validation_count) | add // 0),
          paper_trade_candidate_count:($reports | map(.paper_trade_candidate_count) | add // 0)
        }
    )
  | map(select(.complete == true))
  | sort_by(.source_candidate_count, .last_modified)
  | last // empty_batch
' "$report_records_json")"

research_evidence_summary="$(jq -n -c \
  --argjson latest "$report_summary" \
  --argjson shard_batch "$current_approved_shard_batch_summary" '
    if (
      ($shard_batch.present // false)
      and (($shard_batch.source_candidate_count // 0) > ($latest.source_candidate_count // 0))
    ) then
      $shard_batch + {evidence_source:"current_approved_shard_batch"}
    else
      $latest + {
        evidence_source:"latest_research_report",
        selection:"latest_research_report"
      }
    end
  ')"

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
  --argjson current_approved_shard_batch "$current_approved_shard_batch_summary" \
  --argjson research "$research_evidence_summary" \
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
      research_replay_completed:($research.present and $research.replay_run_count > 0),
      promotion_passed:($research.promotion_bias_count > 0 or $research.shadow_validation_count > 0 or $research.paper_trade_candidate_count > 0),
      shadow_created:($prefixes.shadow_validation_run.key != null),
      paper_created:($prefixes.paper_trade_run.key != null),
      live_enabled:false
    },
    major50_universe:$universe,
    recent_candidates:$candidates,
    latest_research_report:$report,
    best_current_approved_shard_batch:$current_approved_shard_batch,
    research_evidence:$research,
    latest_prefixes:$prefixes,
    coverage_gaps:{
      approved_symbols_without_recent_candidate:(($universe.approved_symbols // []) - ($candidates.distinct_candidate_symbols // [])),
      recent_candidate_symbols_without_replay:(
        if ($research.present and (($research.top_symbols // []) | length) > 0) then
          (($candidates.distinct_candidate_symbols // []) - ($research.top_symbols // []))
        else
          ($candidates.distinct_candidate_symbols // [])
        end
      ),
      replayed_symbols_without_promotion:(
        if ($research.promotion_bias_count == 0) then
          ($research.top_symbols // [])
        else
          []
        end
      )
    },
    next_decision: (
      (($universe.approved_symbols // []) - ($candidates.distinct_candidate_symbols // [])) as $candidate_gap
      | {
        schema_version:"research_loop_state_decision_v1",
        verdict:(
          if ($runtime.runtime_alive | not) then "RUNTIME_NOT_READY"
          elif ($runtime.dispatcher_mode != "run_task") then "AUTO_RESEARCH_DISABLED"
          elif ($universe.major_coverage_complete | not) then "WAIT_FOR_MAJOR50_OBSERVATION"
          elif ($universe.approved_major_coverage_complete | not) then "WAIT_FOR_MAJOR50_APPROVAL"
          elif (($candidate_gap | length) > 0) then "INCREASE_CANDIDATE_GENERATION_COVERAGE"
          elif (($research.present | not) or $research.replay_run_count == 0) then "RUN_RESEARCH_REPLAY"
          elif ($research.promotion_bias_count == 0 and $research.shadow_validation_count == 0) then "ACCUMULATE_RESEARCH_REPLAY_EVIDENCE"
          elif ($prefixes.shadow_validation_run.key == null) then "REVIEW_PROMOTION_FOR_SHADOW"
          elif ($prefixes.paper_trade_run.key == null) then "WAIT_FOR_PASSED_SHADOW_BEFORE_PAPER"
          else "REVIEW_PAPER_PROGRESS"
          end
        ),
        safe_next_actions:([
          if ($runtime.dispatcher_mode != "run_task") then "keep_dispatcher_dry_run_until_output_write_and_duplicate_controls_are_approved" else empty end,
          if ($universe.major_coverage_complete | not) then "wait_for_major50_observation" else empty end,
          if ($universe.approved_major_coverage_complete | not) then "wait_for_major50_approval" else empty end,
          if (($candidate_gap | length) > 0) then "increase_candidate_generation_for_approved_major50_symbols" else empty end,
          if ($research.present and $research.replay_run_count > 0 and $research.promotion_bias_count == 0) then "keep_accumulating_completed_native_replay_samples" else empty end,
          if (($research.present | not) or $research.replay_run_count == 0) then "run_research_replay_for_recent_candidates" else empty end,
          if ($research.promotion_bias_count > 0 and $prefixes.shadow_validation_run.key == null) then "review_promotion_to_shadow_evidence" else empty end
        ]),
        blocked_actions:[
          "do_not_create_shadow_without_promotion",
          "do_not_create_paper_without_completed_passed_shadow",
          "do_not_enable_live_from_loop_state"
        ],
        safety:{
          read_only_check:true,
          s3_write:false,
          ecs_task_started:false,
          dispatcher_mode_changed:false,
          paper_live_enabled:false,
          live_enabled:false,
          order_execution_enabled:false
        },
        evidence:{
          dispatcher_mode:$runtime.dispatcher_mode,
          major50_observed:$universe.major_coverage_complete,
          major50_approved:$universe.approved_major_coverage_complete,
          approved_symbols_without_recent_candidate_count:($candidate_gap | length),
          recent_candidate_symbol_count:$candidates.distinct_candidate_symbol_count,
          research_evidence_source:$research.evidence_source,
          research_replay_count:$research.replay_run_count,
          promotion_bias_count:$research.promotion_bias_count,
          shadow_output_present:($prefixes.shadow_validation_run.key != null),
          paper_output_present:($prefixes.paper_trade_run.key != null)
        }
      }
    ),
    bottlenecks:([
      if ($runtime.dispatcher_mode != "run_task") then "dispatcher_not_run_task" else empty end,
      if ($universe.major_coverage_complete | not) then "major50_observed_universe_incomplete" else empty end,
      if ($universe.approved_major_coverage_complete | not) then "major50_approved_universe_incomplete" else empty end,
      if ($candidates.recent_candidate_record_count == 0) then "no_recent_candidate_bundles" else empty end,
      if (($research.present | not) or $research.replay_run_count == 0) then "research_replay_not_completed" else empty end,
      if ($research.promotion_bias_count == 0 and $research.shadow_validation_count == 0) then "no_promoted_shadow_candidate" else empty end,
      if ($prefixes.shadow_validation_run.key == null) then "shadow_output_absent" else empty end,
      if ($prefixes.paper_trade_run.key == null) then "paper_output_absent" else empty end
    ])
  }' | redact

echo "research loop state check completed"
