use super::*;

mod bundle;
mod manifest;
mod market;
mod replay;
mod validation_runs;

pub(super) fn apply_research_input_arg<I>(
    args: &mut Args,
    arg: &str,
    values: &mut I,
) -> AppResult<bool>
where
    I: Iterator<Item = String>,
{
    if manifest::apply_manifest_arg(args, arg, values)? {
        return Ok(true);
    }
    if bundle::apply_bundle_arg(args, arg, values)? {
        return Ok(true);
    }
    if market::apply_market_arg(args, arg, values)? {
        return Ok(true);
    }
    if replay::apply_replay_arg(args, arg, values)? {
        return Ok(true);
    }
    if validation_runs::apply_validation_run_arg(args, arg, values)? {
        return Ok(true);
    }

    Ok(false)
}
