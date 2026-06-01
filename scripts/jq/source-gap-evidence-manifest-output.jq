include "source-gap-evidence-manifest-selection";

($diagnosis[0]) as $diagnosis_doc
| ($source[0] // {}) as $source_manifest
| (csv_list($focus_statuses)) as $statuses
| source_gap_evidence_refs($diagnosis_doc; $statuses; $candidate_bucket) as $raw_refs
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
        status_counts:source_gap_status_counts($selected_refs),
        primary_blocker_counts:source_gap_primary_blocker_counts($selected_refs),
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
      runtime_budget_policy:source_gap_runtime_budget($selected_refs; $source_manifest)
    }
  }
