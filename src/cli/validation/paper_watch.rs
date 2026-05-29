use crate::cli::Args;
use crate::error::AppResult;

mod candidate;
mod market_live;
mod mode;
mod observer;
mod output;

use candidate::validate_paper_watch_candidate_input;
use market_live::validate_market_live_tick_input;
use mode::validate_paper_watch_live_cycle_mode_is_isolated;
use output::validate_paper_watch_live_cycle_output;

pub(in crate::cli) use observer::validate_paper_watch_observer_args;

pub(in crate::cli) fn validate_paper_watch_live_cycle_args(args: &Args) -> AppResult<()> {
    validate_paper_watch_live_cycle_mode_is_isolated(args)?;
    validate_paper_watch_candidate_input(args)?;
    validate_market_live_tick_input(args)?;
    validate_paper_watch_live_cycle_output(args)?;
    Ok(())
}
