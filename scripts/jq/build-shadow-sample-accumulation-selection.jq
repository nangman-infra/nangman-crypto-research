def unique_sorted: unique | sort;

def candidate_id_from_uri:
  ((.uri // "" | capture("candidate_id=(?<candidate_id>[^/]+)")? // {}) | .candidate_id) // null;

def accumulation_backlog($gap_doc):
  ($gap_doc.shadow_sample_backlog // [])
  | map(select((.sample_deficit // 0) > 0));

def accumulation_status_candidates($status_doc; $backlog_lifecycle_keys):
  [
    $status_doc.candidate_horizon_matrix[]?
    | select(.candidate_lifecycle_key as $key | $key != null and ($backlog_lifecycle_keys | index($key)) != null)
  ]
  | unique_by(.candidate_id)
  | sort_by(.primary_symbol, .candidate_lifecycle_key, .candidate_id);

def accumulation_source_refs($source_manifest):
  ($source_manifest.candidate_bundle_refs // [])
  | map(. + {candidate_id:candidate_id_from_uri});

def selected_accumulation_refs($source_refs; $status_candidate_ids):
  $source_refs
  | map(select(.candidate_id as $id | $id != null and ($status_candidate_ids | index($id)) != null))
  | unique_by(.uri);

def selected_historical_index_refs($source_manifest; $carry_historical_index_refs):
  if $carry_historical_index_refs then
    ($source_manifest.historical_replay_run_index_refs // [])
  else
    []
  end;

def accumulation_backlog_projection($backlog; $status_candidates; $selected_candidate_ids):
  $backlog
  | map(. as $row | {
      candidate_lifecycle_key:$row.candidate_lifecycle_key,
      symbols:($row.symbols // []),
      observed_shadow_run_count:($row.observed_shadow_run_count // 0),
      target_window_materialized_shadow_run_count:($row.target_window_materialized_shadow_run_count // 0),
      required_shadow_sample_count:($row.required_shadow_sample_count // 0),
      sample_requirement_basis:($row.sample_requirement_basis // "target_window_materialized_shadow_run_count"),
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
  | sort_by(-.sample_deficit, .candidate_lifecycle_key);

def accumulation_by_symbol($status_candidates; $selected_candidate_ids):
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
  | sort_by(.symbol);
