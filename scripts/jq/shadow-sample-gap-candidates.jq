def unique_sorted: unique | sort;

def count_status($counts; $status):
  ($counts | map(select(.value == $status) | .count) | add) // 0;

def sample_status:
  .observation_sample_status // {};

def pending_target_window_runs:
  (.runs // [])
  | map(select((.target_window_materialized // false) == false and (.target_exit_deadline_ms // null) != null));

def next_pending_target_exit_deadline_ms:
  (pending_target_window_runs | map(.target_exit_deadline_ms) | min) // null;

def latest_pending_target_exit_deadline_ms:
  (pending_target_window_runs | map(.target_exit_deadline_ms) | max) // null;

def candidate_projection:
  sample_status as $sample
  | (.status_counts // []) as $status_counts
  | {
      candidate_lifecycle_key,
      symbols:(.symbols // []),
      status_counts:$status_counts,
      pending_count:count_status($status_counts; "pending"),
      completed_count:count_status($status_counts; "completed"),
      failed_count:count_status($status_counts; "failed"),
      target_window_materialized_count:(.target_window_materialized_count // 0),
      absolute_window_materialized_count:(.absolute_window_materialized_count // 0),
      observed_shadow_run_count:($sample.observed_shadow_run_count // 0),
      target_window_materialized_shadow_run_count:($sample.target_window_materialized_shadow_run_count // 0),
      pending_target_window_shadow_run_count:(pending_target_window_runs | length),
      next_pending_target_exit_deadline_ms:next_pending_target_exit_deadline_ms,
      latest_pending_target_exit_deadline_ms:latest_pending_target_exit_deadline_ms,
      required_shadow_sample_count:($sample.required_shadow_sample_count // 0),
      sample_requirement_basis:($sample.sample_requirement_basis // "target_window_materialized_shadow_run_count"),
      sample_requirement_met:($sample.sample_requirement_met // false),
      sample_deficit:($sample.sample_deficit // 0),
      recommended_action:(
        if (($sample.sample_requirement_met // false) == true) then "review_shadow_completion_evidence"
        elif (($sample.target_window_materialized_shadow_run_count // 0) == 0) then "wait_for_target_holding_window"
        elif (($sample.target_window_materialized_shadow_run_count // 0) < ($sample.observed_shadow_run_count // 0)) then "wait_for_pending_shadow_target_window_materialization"
        elif (($sample.sample_deficit // 0) > 0) then "accumulate_shadow_observation_samples"
        else "review_shadow_completion_evidence" end
      )
    };

def shadow_sample_gap_candidates:
  (.by_candidate_lifecycle_key // []) | map(candidate_projection);
