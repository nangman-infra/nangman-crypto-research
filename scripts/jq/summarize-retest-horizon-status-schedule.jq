include "summarize-retest-horizon-status-lib";

def materialization_schedule($rows; $latest_l1_as_of_ms):
  ($rows | min_ms_for_action("wait_for_market_l1_horizon"; "horizon_due_ms")) as $next_wait_due_ms
  | ($rows | max_ms_for_action("wait_for_market_l1_horizon"; "horizon_due_ms")) as $last_wait_due_ms
  | ($rows | min_ms_for_action("accumulate_completed_native_replay_samples"; "horizon_due_ms")) as $oldest_accumulation_due_ms
  | ($rows | max_ms_for_action("accumulate_completed_native_replay_samples"; "horizon_due_ms")) as $latest_accumulation_due_ms
  | {
      latest_l1_as_of_ms:$latest_l1_as_of_ms,
      latest_l1_as_of_iso:iso_ms($latest_l1_as_of_ms),
      next_wait_horizon_due_ms:$next_wait_due_ms,
      next_wait_horizon_due_iso:iso_ms($next_wait_due_ms),
      last_wait_horizon_due_ms:$last_wait_due_ms,
      last_wait_horizon_due_iso:iso_ms($last_wait_due_ms),
      next_wait_deficit_ms:(
        if $latest_l1_as_of_ms == null or $next_wait_due_ms == null then null
        else ([($next_wait_due_ms - $latest_l1_as_of_ms), 0] | max)
        end
      ),
      oldest_accumulation_due_ms:$oldest_accumulation_due_ms,
      oldest_accumulation_due_iso:iso_ms($oldest_accumulation_due_ms),
      latest_accumulation_due_ms:$latest_accumulation_due_ms,
      latest_accumulation_due_iso:iso_ms($latest_accumulation_due_ms)
    };

def next_wait_due_ms($rows):
  $rows | min_ms_for_action("wait_for_market_l1_horizon"; "horizon_due_ms");
