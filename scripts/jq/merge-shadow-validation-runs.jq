def unique_sorted: unique | sort;

def records:
  map(if type == "array" then .[] else . end)
  | map(select(type == "object"));

def status_value: (.status // "pending");

def counts_by(expr):
  map(expr)
  | sort
  | group_by(.)
  | map({value:.[0], count:length});

records as $runs
| (
    $runs
    | map(select((.shadow_validation_run_id // "") != ""))
    | group_by(.shadow_validation_run_id)
    | map(last)
    | sort_by(.candidate_lifecycle_key // "", .symbol_canonical // "", .shadow_validation_run_id // "")
  ) as $merged
| (
    $runs
    | map(select((.shadow_validation_run_id // "") != ""))
    | group_by(.shadow_validation_run_id)
    | map(select(length > 1) | {
        shadow_validation_run_id:.[0].shadow_validation_run_id,
        duplicate_count:length,
        statuses:(map(status_value) | unique_sorted),
        passed_values:(map(.passed // false) | unique | sort)
      })
  ) as $duplicates
| {
    summary:{
      schema_version:"research_shadow_validation_merge_summary_v1",
      generated_at:$generated_at,
      generated_at_ms:$generated_at_ms,
      output_file:$output_file,
      summary_output:$summary_output,
      input_files:($input_files[0] // []),
      safety:{
        s3_write:false,
        ecs_task_started:false,
        dispatcher_mode_changed:false,
        local_merge_only:true,
        shadow_status_mutated:false,
        paper_live_enabled:false
      },
      input_record_count:($runs | length),
      merged_record_count:($merged | length),
      duplicate_record_count:(($runs | length) - ($merged | length)),
      duplicate_shadow_validation_run_count:($duplicates | length),
      schema_versions:($merged | map(.schema_version // "unknown") | unique_sorted),
      status_counts:($merged | counts_by(status_value)),
      symbol_count:($merged | map(.symbol_canonical // empty) | unique | length),
      symbols:($merged | map(.symbol_canonical // empty) | unique_sorted),
      candidate_lifecycle_count:($merged | map(.candidate_lifecycle_key // empty) | unique | length),
      duplicate_shadow_validation_runs:$duplicates,
      blocked_actions:[
        "do_not_mark_pending_shadow_passed_from_merge",
        "do_not_create_paper_without_completed_passed_shadow",
        "do_not_enable_live_from_shadow_merge"
      ]
    },
    records:$merged
  }
