#!/usr/bin/env bash

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
