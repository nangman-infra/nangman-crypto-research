include "build-focused-retest-manifest-common";

def focused_retest_rows($status_doc; $actions):
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
  | sort_by(.focus_symbol, .candidate_id, (.horizon | horizon_order));

def carry_historical_index_refs($include_historical_index_refs; $actions):
  $include_historical_index_refs == "true"
  or (
    $include_historical_index_refs == "auto"
    and ($actions | index("accumulate_completed_native_replay_samples")) != null
  );

def source_candidate_refs($source_manifest):
  ($source_manifest.candidate_bundle_refs // [])
  | map(. + {candidate_id:candidate_id_from_uri});

def selected_candidate_refs($source_refs; $focus_candidate_ids):
  $source_refs
  | map(select(.candidate_id as $candidate_id | $candidate_id != null and ($focus_candidate_ids | index($candidate_id)) != null))
  | unique_by(.uri);

def selected_historical_index_refs($source_manifest; $carry_historical_index_refs):
  if $carry_historical_index_refs then
    ($source_manifest.historical_replay_run_index_refs // [])
  else
    []
  end;
