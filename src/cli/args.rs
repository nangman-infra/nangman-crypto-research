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

mod defaults;
mod parser;
mod types;
mod validation_flow;

use defaults::args_from_env;
use parser::{CliArgsOutcome, apply_cli_args};
pub use types::Args;
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
