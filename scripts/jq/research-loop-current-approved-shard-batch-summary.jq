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
