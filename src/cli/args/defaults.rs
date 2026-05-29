use super::*;

pub(super) fn args_from_env() -> AppResult<Args> {
    let shadow_cycle_latest_l1_as_of_ms =
        env_non_negative_i64("RESEARCH_SHADOW_CYCLE_LATEST_L1_AS_OF_MS")?;
    let retest_horizon_latest_l1_as_of_ms =
        env_non_negative_i64("RESEARCH_RETEST_PLAN_LATEST_L1_AS_OF_MS")?;
    let focused_retest_historical_replay_index_ref_mode =
        env_string("RESEARCH_FOCUS_INCLUDE_HISTORICAL_INDEX_REFS")
            .map(|value| HistoricalReplayIndexRefMode::parse(&value))
            .transpose()?
            .unwrap_or(HistoricalReplayIndexRefMode::Auto);
    let focused_retest_next_actions = env_string("RESEARCH_FOCUS_NEXT_ACTIONS")
        .map(|value| parse_focused_retest_actions(&value))
        .filter(|actions| !actions.is_empty())
        .unwrap_or_else(default_focused_retest_actions);
    Ok(Args {
        build_shadow_cycle_decision: env_bool("RESEARCH_BUILD_SHADOW_CYCLE_DECISION"),
        run_shadow_cycle_from_latest_state: env_bool("RESEARCH_RUN_SHADOW_CYCLE_FROM_LATEST_STATE"),
        build_retest_horizon_plan: env_bool("RESEARCH_BUILD_RETEST_HORIZON_PLAN"),
        run_retest_refresh_cycle: env_bool("RESEARCH_RUN_RETEST_REFRESH_CYCLE"),
        run_retest_refresh_cycle_from_latest_state: env_bool(
            "RESEARCH_RUN_RETEST_REFRESH_CYCLE_FROM_LATEST_STATE",
        ),
        run_retest_cycle_scheduler: env_bool("RESEARCH_RUN_RETEST_CYCLE_SCHEDULER"),
        build_retest_horizon_status: env_bool("RESEARCH_BUILD_RETEST_HORIZON_STATUS"),
        build_focused_retest_manifest: env_bool("RESEARCH_BUILD_FOCUSED_RETEST_MANIFEST"),
        run_paper_watch_live_cycle: env_bool("RESEARCH_RUN_PAPER_WATCH_LIVE_CYCLE"),
        run_paper_watch_observer: env_bool("RESEARCH_RUN_PAPER_WATCH_OBSERVER"),
        shadow_cycle_decision_file: None,
        shadow_cycle_decision_output_file: env_string("RESEARCH_SHADOW_CYCLE_DECISION_OUTPUT_FILE")
            .map(PathBuf::from),
        shadow_cycle_latest_l1_as_of_ms,
        retest_horizon_plan_file: env_string("RESEARCH_RETEST_HORIZON_PLAN_FILE")
            .map(PathBuf::from),
        retest_horizon_plan_s3_bucket: env_string("RESEARCH_RETEST_HORIZON_PLAN_S3_BUCKET"),
        retest_horizon_plan_s3_key: env_string("RESEARCH_RETEST_HORIZON_PLAN_S3_KEY"),
        retest_horizon_plan_output_file: env_string("RESEARCH_RETEST_HORIZON_PLAN_OUTPUT_FILE")
            .map(PathBuf::from),
        retest_horizon_latest_l1_as_of_ms,
        retest_horizon_status_output_file: env_string("RESEARCH_RETEST_HORIZON_STATUS_OUTPUT_FILE")
            .map(PathBuf::from),
        retest_driver_summary_file: env_string("RESEARCH_RETEST_DRIVER_SUMMARY_FILE")
            .map(PathBuf::from),
        retest_horizon_status_file: env_string("RESEARCH_HORIZON_STATUS_FILE").map(PathBuf::from),
        retest_horizon_status_s3_bucket: env_string("RESEARCH_HORIZON_STATUS_S3_BUCKET"),
        retest_horizon_status_s3_key: env_string("RESEARCH_HORIZON_STATUS_S3_KEY"),
        focused_retest_manifest_output_file: env_string("RESEARCH_FOCUS_MANIFEST_OUTPUT")
            .map(PathBuf::from),
        focused_retest_summary_output_file: env_string("RESEARCH_FOCUS_SUMMARY_OUTPUT")
            .map(PathBuf::from),
        focused_retest_next_actions,
        focused_retest_historical_replay_index_ref_mode,
        input_manifest_file: None,
        input_manifest_s3_bucket: env_string("RESEARCH_INPUT_MANIFEST_S3_BUCKET"),
        input_manifest_s3_key: env_string("RESEARCH_INPUT_MANIFEST_S3_KEY"),
        research_report_file: env_string("RESEARCH_REPORT_FILE").map(PathBuf::from),
        research_report_s3_bucket: env_string("RESEARCH_REPORT_S3_BUCKET"),
        research_report_s3_key: env_string("RESEARCH_REPORT_S3_KEY"),
        input_bundle_file: None,
        input_bundle_s3_bucket: env_string("RESEARCH_INPUT_S3_BUCKET"),
        input_bundle_s3_key: env_string("RESEARCH_INPUT_S3_KEY"),
        market_feature_delta_file: None,
        market_regime_context_file: None,
        market_l1_s3_bucket: env_string("RESEARCH_MARKET_L1_S3_BUCKET"),
        market_feature_delta_s3_keys: env_list("RESEARCH_MARKET_FEATURE_DELTA_S3_KEYS"),
        market_regime_context_s3_keys: env_list("RESEARCH_MARKET_REGIME_CONTEXT_S3_KEYS"),
        historical_replay_run_files: Vec::new(),
        historical_replay_run_index_files: Vec::new(),
        oss_adapter_run_files: Vec::new(),
        shadow_validation_run_files: Vec::new(),
        oss_adapter_run_s3_bucket: env_string("RESEARCH_OSS_ADAPTER_RUN_S3_BUCKET"),
        oss_adapter_run_s3_keys: env_list("RESEARCH_OSS_ADAPTER_RUN_S3_KEYS"),
        shadow_validation_run_s3_bucket: env_string("RESEARCH_SHADOW_VALIDATION_RUN_S3_BUCKET"),
        shadow_validation_run_s3_keys: env_list("RESEARCH_SHADOW_VALIDATION_RUN_S3_KEYS"),
        paper_watch_candidate_file: env_string("RESEARCH_PAPER_WATCH_CANDIDATE_FILE")
            .map(PathBuf::from),
        paper_watch_candidate_s3_bucket: env_string("RESEARCH_PAPER_WATCH_CANDIDATE_S3_BUCKET"),
        paper_watch_candidate_s3_key: env_string("RESEARCH_PAPER_WATCH_CANDIDATE_S3_KEY"),
        paper_watch_candidate_s3_prefix: env_string("RESEARCH_PAPER_WATCH_CANDIDATE_S3_PREFIX")
            .unwrap_or_else(|| DEFAULT_PAPER_WATCH_CANDIDATE_PREFIX.to_owned()),
        paper_watch_observer_read_limit: env_usize(
            "RESEARCH_PAPER_WATCH_OBSERVER_READ_LIMIT",
            DEFAULT_PAPER_WATCH_OBSERVER_READ_LIMIT,
        )?,
        paper_watch_observer_scan_limit: env_usize(
            "RESEARCH_PAPER_WATCH_OBSERVER_SCAN_LIMIT",
            DEFAULT_PAPER_WATCH_OBSERVER_SCAN_LIMIT,
        )?,
        paper_watch_observer_poll_secs: env_u64(
            "RESEARCH_PAPER_WATCH_OBSERVER_POLL_SECS",
            DEFAULT_PAPER_WATCH_OBSERVER_POLL_SECS,
        )?,
        paper_watch_observer_max_iterations: env_usize_allow_zero(
            "RESEARCH_PAPER_WATCH_OBSERVER_MAX_ITERATIONS",
            0,
        )?,
        paper_watch_live_mark_s3_prefix: env_string("RESEARCH_PAPER_WATCH_LIVE_MARK_S3_PREFIX")
            .unwrap_or_else(|| DEFAULT_PAPER_WATCH_LIVE_MARK_PREFIX.to_owned()),
        paper_watch_live_mark_read_limit: env_usize(
            "RESEARCH_PAPER_WATCH_LIVE_MARK_READ_LIMIT",
            DEFAULT_PAPER_WATCH_OBSERVER_READ_LIMIT,
        )?,
        paper_watch_live_mark_scan_limit: env_usize(
            "RESEARCH_PAPER_WATCH_LIVE_MARK_SCAN_LIMIT",
            DEFAULT_PAPER_WATCH_OBSERVER_SCAN_LIMIT,
        )?,
        market_live_tick_file: env_string("RESEARCH_MARKET_LIVE_TICK_FILE").map(PathBuf::from),
        market_live_nats_url: env_string("RESEARCH_MARKET_LIVE_NATS_URL"),
        market_live_nats_stream: env_string("RESEARCH_MARKET_LIVE_NATS_STREAM")
            .unwrap_or_else(|| DEFAULT_MARKET_LIVE_NATS_STREAM.to_owned()),
        market_live_nats_subject: env_string("RESEARCH_MARKET_LIVE_NATS_SUBJECT")
            .unwrap_or_else(|| DEFAULT_MARKET_LIVE_NATS_SUBJECT.to_owned()),
        market_live_nats_consumer: env_string("RESEARCH_MARKET_LIVE_NATS_CONSUMER")
            .unwrap_or_else(|| DEFAULT_MARKET_LIVE_NATS_CONSUMER.to_owned()),
        market_live_nats_deliver_policy: env_string("RESEARCH_MARKET_LIVE_NATS_DELIVER_POLICY")
            .unwrap_or_else(|| DEFAULT_MARKET_LIVE_NATS_DELIVER_POLICY.to_owned()),
        market_live_nats_batch_size: env_usize(
            "RESEARCH_MARKET_LIVE_NATS_BATCH_SIZE",
            DEFAULT_MARKET_LIVE_NATS_BATCH_SIZE,
        )?,
        market_live_nats_max_messages: env_usize(
            "RESEARCH_MARKET_LIVE_NATS_MAX_MESSAGES",
            DEFAULT_MARKET_LIVE_NATS_MAX_MESSAGES,
        )?,
        market_live_nats_ack_wait_secs: env_u64(
            "RESEARCH_MARKET_LIVE_NATS_ACK_WAIT_SECS",
            DEFAULT_MARKET_LIVE_NATS_ACK_WAIT_SECS,
        )?,
        historical_replay_run_s3_bucket: env_string("RESEARCH_HISTORICAL_REPLAY_RUN_S3_BUCKET"),
        historical_replay_run_s3_keys: env_list("RESEARCH_HISTORICAL_REPLAY_RUN_S3_KEYS"),
        historical_replay_run_index_s3_bucket: env_string(
            "RESEARCH_HISTORICAL_REPLAY_RUN_INDEX_S3_BUCKET",
        ),
        historical_replay_run_index_s3_keys: env_list(
            "RESEARCH_HISTORICAL_REPLAY_RUN_INDEX_S3_KEYS",
        ),
        output_dir: None,
        output_s3_bucket: env_string("RESEARCH_OUTPUT_S3_BUCKET"),
        output_s3_prefix: env_string("RESEARCH_OUTPUT_S3_PREFIX"),
        research_packet_id: "local_research_packet".to_owned(),
        run_scope: "p0_candidate_bundle_local".to_owned(),
        now_ms: None,
    })
}
