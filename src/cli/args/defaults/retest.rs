use super::*;

pub(super) fn apply_retest_env(args: &mut Args) -> AppResult<()> {
    args.shadow_cycle_decision_output_file =
        env_string("RESEARCH_SHADOW_CYCLE_DECISION_OUTPUT_FILE").map(PathBuf::from);
    args.shadow_cycle_latest_l1_as_of_ms =
        env_non_negative_i64("RESEARCH_SHADOW_CYCLE_LATEST_L1_AS_OF_MS")?;
    args.retest_horizon_plan_file =
        env_string("RESEARCH_RETEST_HORIZON_PLAN_FILE").map(PathBuf::from);
    args.retest_horizon_plan_s3_bucket = env_string("RESEARCH_RETEST_HORIZON_PLAN_S3_BUCKET");
    args.retest_horizon_plan_s3_key = env_string("RESEARCH_RETEST_HORIZON_PLAN_S3_KEY");
    args.retest_horizon_plan_output_file =
        env_string("RESEARCH_RETEST_HORIZON_PLAN_OUTPUT_FILE").map(PathBuf::from);
    args.retest_horizon_latest_l1_as_of_ms =
        env_non_negative_i64("RESEARCH_RETEST_PLAN_LATEST_L1_AS_OF_MS")?;
    args.retest_horizon_status_output_file =
        env_string("RESEARCH_RETEST_HORIZON_STATUS_OUTPUT_FILE").map(PathBuf::from);
    args.retest_driver_summary_file =
        env_string("RESEARCH_RETEST_DRIVER_SUMMARY_FILE").map(PathBuf::from);
    args.retest_horizon_status_file = env_string("RESEARCH_HORIZON_STATUS_FILE").map(PathBuf::from);
    args.retest_horizon_status_s3_bucket = env_string("RESEARCH_HORIZON_STATUS_S3_BUCKET");
    args.retest_horizon_status_s3_key = env_string("RESEARCH_HORIZON_STATUS_S3_KEY");
    args.focused_retest_manifest_output_file =
        env_string("RESEARCH_FOCUS_MANIFEST_OUTPUT").map(PathBuf::from);
    args.focused_retest_summary_output_file =
        env_string("RESEARCH_FOCUS_SUMMARY_OUTPUT").map(PathBuf::from);
    args.focused_retest_historical_replay_index_ref_mode =
        focused_replay_index_ref_mode_from_env()?;
    args.focused_retest_next_actions = focused_retest_next_actions_from_env();
    Ok(())
}

fn focused_replay_index_ref_mode_from_env() -> AppResult<HistoricalReplayIndexRefMode> {
    Ok(env_string("RESEARCH_FOCUS_INCLUDE_HISTORICAL_INDEX_REFS")
        .map(|value| HistoricalReplayIndexRefMode::parse(&value))
        .transpose()?
        .unwrap_or(HistoricalReplayIndexRefMode::Auto))
}

fn focused_retest_next_actions_from_env() -> Vec<String> {
    env_string("RESEARCH_FOCUS_NEXT_ACTIONS")
        .map(|value| parse_focused_retest_actions(&value))
        .filter(|actions| !actions.is_empty())
        .unwrap_or_else(default_focused_retest_actions)
}
