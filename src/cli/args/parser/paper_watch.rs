use super::*;

mod candidate;
mod live_mark;
mod market_live;
mod observer;

pub(super) fn apply_paper_watch_arg<I>(
    args: &mut Args,
    arg: &str,
    values: &mut I,
) -> AppResult<bool>
where
    I: Iterator<Item = String>,
{
    if candidate::apply_candidate_arg(args, arg, values)? {
        return Ok(true);
    }
    if observer::apply_observer_arg(args, arg, values)? {
        return Ok(true);
    }
    if live_mark::apply_live_mark_arg(args, arg, values)? {
        return Ok(true);
    }
    if market_live::apply_market_live_arg(args, arg, values)? {
        return Ok(true);
    }

    Ok(false)
}
