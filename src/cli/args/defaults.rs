use super::*;

mod base;
mod inputs;
mod market_live;
mod modes;
mod output;
mod paper_watch;
mod retest;

pub(super) fn args_from_env() -> AppResult<Args> {
    let mut args = Args::default();
    modes::apply_mode_env(&mut args);
    retest::apply_retest_env(&mut args)?;
    inputs::apply_research_input_env(&mut args);
    paper_watch::apply_paper_watch_env(&mut args)?;
    market_live::apply_market_live_env(&mut args)?;
    output::apply_history_and_output_env(&mut args);
    Ok(args)
}
