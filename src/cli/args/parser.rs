use super::*;

mod common;
mod inputs;
mod modes;
mod paper_watch;
mod retest;

use common::apply_common_arg;
use inputs::apply_research_input_arg;
use modes::apply_mode_flag;
use paper_watch::apply_paper_watch_arg;
use retest::apply_retest_arg;

pub(super) enum CliArgsOutcome {
    Continue,
    Help,
}

pub(super) fn apply_cli_args<I>(args: &mut Args, mut values: I) -> AppResult<CliArgsOutcome>
where
    I: Iterator<Item = String>,
{
    while let Some(arg) = values.next() {
        if matches!(arg.as_str(), "-h" | "--help") {
            return Ok(CliArgsOutcome::Help);
        }
        if apply_mode_flag(args, &arg) {
            continue;
        }
        if apply_retest_arg(args, &arg, &mut values)? {
            continue;
        }
        if apply_research_input_arg(args, &arg, &mut values)? {
            continue;
        }
        if apply_paper_watch_arg(args, &arg, &mut values)? {
            continue;
        }
        if apply_common_arg(args, &arg, &mut values)? {
            continue;
        }
        return Err(AppError::config(format!(
            "unknown argument: {arg}\n\n{}",
            help_text()
        )));
    }

    Ok(CliArgsOutcome::Continue)
}
