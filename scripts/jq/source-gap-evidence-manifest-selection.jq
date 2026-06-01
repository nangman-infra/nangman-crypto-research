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

def source_gap_evidence_refs($diagnosis_doc; $statuses; $candidate_bucket):
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
  ];

def source_gap_status_counts($selected_refs):
  $selected_refs
  | sort_by(.status)
  | group_by(.status)
  | map({status:.[0].status, count:length})
  | sort_by(.count, .status)
  | reverse;

def source_gap_primary_blocker_counts($selected_refs):
  $selected_refs
  | map(select(.primary_blocker != null))
  | sort_by(.primary_blocker)
  | group_by(.primary_blocker)
  | map({primary_blocker:.[0].primary_blocker, count:length})
  | sort_by(.count, .primary_blocker)
  | reverse;

def source_gap_runtime_budget($selected_refs; $source_manifest):
  default_runtime_budget($selected_refs | length)
  + ($source_manifest.runtime_budget_policy // {})
  + {
      max_candidate_bundle_count:(
        if ($selected_refs | length) > 0 then ($selected_refs | length) else 1 end
      )
    };
