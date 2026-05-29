use super::{
    DEFAULT_PAPER_WATCH_CANDIDATE_PREFIX, DEFAULT_PAPER_WATCH_LIVE_MARK_PREFIX,
    DEFAULT_PAPER_WATCH_OBSERVER_POLL_SECS, DEFAULT_PAPER_WATCH_OBSERVER_READ_LIMIT,
    DEFAULT_PAPER_WATCH_OBSERVER_SCAN_LIMIT, absolute_path_arg, env_bool, env_list,
    env_non_negative_i64, env_string, env_u64, env_usize, env_usize_allow_zero,
    has_retest_horizon_status_input, help_text, non_empty_arg, parse_non_negative_i64,
    parse_non_negative_usize, parse_positive_u64, parse_positive_usize,
    validate_focused_retest_manifest_build_args, validate_paper_watch_live_cycle_args,
    validate_paper_watch_observer_args, validate_research_report_input_args,
    validate_retest_cycle_scheduler_args, validate_retest_horizon_plan_build_args,
    validate_retest_horizon_plan_input_args, validate_retest_horizon_status_build_args,
    validate_retest_horizon_status_input_args, validate_retest_refresh_cycle_args,
    validate_retest_refresh_cycle_from_latest_state_args, validate_shadow_cycle_build_args,
    validate_shadow_cycle_from_latest_state_args,
};
use crate::error::{AppError, AppResult};
use crate::focused_retest::{
    HistoricalReplayIndexRefMode, default_focused_retest_actions, parse_focused_retest_actions,
};
use crate::paper_live::{
    DEFAULT_MARKET_LIVE_NATS_ACK_WAIT_SECS, DEFAULT_MARKET_LIVE_NATS_BATCH_SIZE,
    DEFAULT_MARKET_LIVE_NATS_CONSUMER, DEFAULT_MARKET_LIVE_NATS_DELIVER_POLICY,
    DEFAULT_MARKET_LIVE_NATS_MAX_MESSAGES, DEFAULT_MARKET_LIVE_NATS_STREAM,
    DEFAULT_MARKET_LIVE_NATS_SUBJECT,
};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Args {
    pub build_shadow_cycle_decision: bool,
    pub run_shadow_cycle_from_latest_state: bool,
    pub build_retest_horizon_plan: bool,
    pub run_retest_refresh_cycle: bool,
    pub run_retest_refresh_cycle_from_latest_state: bool,
    pub run_retest_cycle_scheduler: bool,
    pub build_retest_horizon_status: bool,
    pub build_focused_retest_manifest: bool,
    pub run_paper_watch_live_cycle: bool,
    pub run_paper_watch_observer: bool,
    pub shadow_cycle_decision_file: Option<PathBuf>,
    pub shadow_cycle_decision_output_file: Option<PathBuf>,
    pub shadow_cycle_latest_l1_as_of_ms: Option<i64>,
    pub retest_horizon_plan_file: Option<PathBuf>,
    pub retest_horizon_plan_s3_bucket: Option<String>,
    pub retest_horizon_plan_s3_key: Option<String>,
    pub retest_horizon_plan_output_file: Option<PathBuf>,
    pub retest_horizon_latest_l1_as_of_ms: Option<i64>,
    pub retest_horizon_status_output_file: Option<PathBuf>,
    pub retest_driver_summary_file: Option<PathBuf>,
    pub retest_horizon_status_file: Option<PathBuf>,
    pub retest_horizon_status_s3_bucket: Option<String>,
    pub retest_horizon_status_s3_key: Option<String>,
    pub focused_retest_manifest_output_file: Option<PathBuf>,
    pub focused_retest_summary_output_file: Option<PathBuf>,
    pub focused_retest_next_actions: Vec<String>,
    pub focused_retest_historical_replay_index_ref_mode: HistoricalReplayIndexRefMode,
    pub input_manifest_file: Option<PathBuf>,
    pub input_manifest_s3_bucket: Option<String>,
    pub input_manifest_s3_key: Option<String>,
    pub research_report_file: Option<PathBuf>,
    pub research_report_s3_bucket: Option<String>,
    pub research_report_s3_key: Option<String>,
    pub input_bundle_file: Option<PathBuf>,
    pub input_bundle_s3_bucket: Option<String>,
    pub input_bundle_s3_key: Option<String>,
    pub market_feature_delta_file: Option<PathBuf>,
    pub market_regime_context_file: Option<PathBuf>,
    pub market_l1_s3_bucket: Option<String>,
    pub market_feature_delta_s3_keys: Vec<String>,
    pub market_regime_context_s3_keys: Vec<String>,
    pub historical_replay_run_files: Vec<PathBuf>,
    pub historical_replay_run_index_files: Vec<PathBuf>,
    pub oss_adapter_run_files: Vec<PathBuf>,
    pub shadow_validation_run_files: Vec<PathBuf>,
    pub oss_adapter_run_s3_bucket: Option<String>,
    pub oss_adapter_run_s3_keys: Vec<String>,
    pub shadow_validation_run_s3_bucket: Option<String>,
    pub shadow_validation_run_s3_keys: Vec<String>,
    pub paper_watch_candidate_file: Option<PathBuf>,
    pub paper_watch_candidate_s3_bucket: Option<String>,
    pub paper_watch_candidate_s3_key: Option<String>,
    pub paper_watch_candidate_s3_prefix: String,
    pub paper_watch_observer_read_limit: usize,
    pub paper_watch_observer_scan_limit: usize,
    pub paper_watch_observer_poll_secs: u64,
    pub paper_watch_observer_max_iterations: usize,
    pub paper_watch_live_mark_s3_prefix: String,
    pub paper_watch_live_mark_read_limit: usize,
    pub paper_watch_live_mark_scan_limit: usize,
    pub market_live_tick_file: Option<PathBuf>,
    pub market_live_nats_url: Option<String>,
    pub market_live_nats_stream: String,
    pub market_live_nats_subject: String,
    pub market_live_nats_consumer: String,
    pub market_live_nats_deliver_policy: String,
    pub market_live_nats_batch_size: usize,
    pub market_live_nats_max_messages: usize,
    pub market_live_nats_ack_wait_secs: u64,
    pub historical_replay_run_s3_bucket: Option<String>,
    pub historical_replay_run_s3_keys: Vec<String>,
    pub historical_replay_run_index_s3_bucket: Option<String>,
    pub historical_replay_run_index_s3_keys: Vec<String>,
    pub output_dir: Option<PathBuf>,
    pub output_s3_bucket: Option<String>,
    pub output_s3_prefix: Option<String>,
    pub research_packet_id: String,
    pub run_scope: String,
    pub now_ms: Option<i64>,
}

mod defaults;
mod parser;
mod validation_flow;

use defaults::args_from_env;
use parser::{CliArgsOutcome, apply_cli_args};
use validation_flow::validate_args;

pub fn parse_args<I>(values: I) -> AppResult<Option<Args>>
where
    I: Iterator<Item = String>,
{
    let mut args = args_from_env()?;
    if matches!(apply_cli_args(&mut args, values)?, CliArgsOutcome::Help) {
        return Ok(None);
    }
    validate_args(&args)?;
    Ok(Some(args))
}
