import "build-shadow-sample-accumulation-selection" as selection;

($gap[0]) as $gap_doc
| ($status[0]) as $status_doc
| ($source[0]) as $source_manifest
| ($include_historical_index_refs == "true") as $carry_historical_index_refs
| selection::accumulation_backlog($gap_doc) as $backlog
| ($backlog | map(.candidate_lifecycle_key) | selection::unique_sorted) as $backlog_lifecycle_keys
| selection::accumulation_status_candidates($status_doc; $backlog_lifecycle_keys) as $status_candidates
| ($status_candidates | map(.candidate_id) | selection::unique_sorted) as $status_candidate_ids
| selection::accumulation_source_refs($source_manifest) as $source_refs
| selection::selected_accumulation_refs($source_refs; $status_candidate_ids) as $selected_refs
| ($selected_refs | map(.candidate_id) | selection::unique_sorted) as $selected_candidate_ids
| selection::selected_historical_index_refs($source_manifest; $carry_historical_index_refs) as $selected_historical_index_refs
| selection::accumulation_backlog_projection($backlog; $status_candidates; $selected_candidate_ids) as $backlog_projection
| {
    summary:{
      schema_version:"research_shadow_sample_accumulation_manifest_summary_v1",
      generated_at:$generated_at,
      generated_at_ms:($generated_at_ms | tonumber),
      shadow_sample_gap_manifest_file:$gap_manifest_file,
      retest_horizon_status_file:$horizon_status_file,
      source_manifest_file:$source_manifest_file,
      accumulation_manifest_output:$accumulation_manifest_output,
      accumulation_summary_output:$accumulation_summary_output,
      safety:{
        s3_write:false,
        ecs_task_started:false,
        dispatcher_mode_changed:false,
        local_manifest_only:true,
        shadow_status_mutated:false,
        paper_live_enabled:false,
        selected_from_existing_source_manifest:true,
        historical_replay_run_index_refs_carried:$carry_historical_index_refs
      },
      source_state:{
        gap_manifest_schema_version:($gap_doc.schema_version // null),
        gap_manifest_verdict:($gap_doc.next_decision.verdict // null),
        retest_horizon_status_schema_version:($status_doc.schema_version // null),
        retest_horizon_verdict:($status_doc.verdict // null),
        source_research_packet_id:($source_manifest.research_packet_id // null),
        source_run_scope:($source_manifest.run_scope // null),
        source_candidate_bundle_ref_count:(($source_manifest.candidate_bundle_refs // []) | length),
        source_historical_replay_run_index_ref_count:(($source_manifest.historical_replay_run_index_refs // []) | length)
      },
      backlog_summary:{
        backlog_candidate_lifecycle_count:($backlog_lifecycle_keys | length),
        backlog_symbol_count:($backlog | map(.symbols // []) | flatten | unique | length),
        backlog_symbols:($backlog | map(.symbols // []) | flatten | selection::unique_sorted),
        total_sample_deficit:(($backlog | map(.sample_deficit // 0) | add) // 0),
        largest_sample_deficit:(($backlog | map(.sample_deficit // 0) | max) // 0),
        pending_lifecycle_count:($backlog | map(select((.pending_count // 0) > 0)) | length),
        status_candidate_count:($status_candidates | length),
        selected_candidate_bundle_ref_count:($selected_refs | length),
        selected_historical_replay_run_index_ref_count:($selected_historical_index_refs | length),
        missing_candidate_ref_count:(($status_candidate_ids - $selected_candidate_ids) | length)
      },
      next_decision:{
        verdict:(
          if ($backlog | length) == 0 then "NO_SHADOW_SAMPLE_BACKLOG"
          elif ($status_candidates | length) == 0 then "NO_STATUS_CANDIDATES_FOR_BACKLOG"
          elif ($selected_refs | length) == 0 then "NO_SOURCE_MANIFEST_REFS_FOR_BACKLOG"
          else "RUN_FOCUSED_SHADOW_SAMPLE_ACCUMULATION_RESEARCH" end
        ),
        safe_next_actions:[
          if ($selected_refs | length) > 0 then "run_research_with_shadow_accumulation_manifest" else empty end,
          if ($selected_refs | length) > 0 then "recompute_shadow_observation_plan_after_research" else empty end,
          if ($selected_refs | length) > 0 then "recompute_shadow_sample_gap_manifest_after_research" else empty end,
          if (($status_candidate_ids - $selected_candidate_ids) | length) > 0 then "inspect_missing_candidate_bundle_refs" else empty end
        ],
        blocked_actions:[
          "do_not_mark_pending_shadow_passed_from_accumulation_manifest",
          "do_not_create_paper_without_completed_passed_shadow",
          "do_not_enable_live_from_shadow_accumulation_manifest"
        ]
      },
      shadow_sample_backlog:$backlog_projection,
      selected_candidate_ids:$selected_candidate_ids,
      missing_candidate_ref_ids:($status_candidate_ids - $selected_candidate_ids),
      by_symbol:selection::accumulation_by_symbol($status_candidates; $selected_candidate_ids)
    },
    manifest:{
      schema_version:($source_manifest.schema_version // "research_input_manifest_v1"),
      research_packet_id:$accumulation_packet_id,
      run_scope:$accumulation_run_scope,
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
