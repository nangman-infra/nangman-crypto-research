use super::*;

mod focused;
mod plan;
mod shadow;
mod status;

pub(super) fn apply_retest_arg<I>(args: &mut Args, arg: &str, values: &mut I) -> AppResult<bool>
where
    I: Iterator<Item = String>,
{
    if shadow::apply_shadow_cycle_arg(args, arg, values)? {
        return Ok(true);
    }
    if plan::apply_retest_plan_arg(args, arg, values)? {
        return Ok(true);
    }
    if status::apply_retest_status_arg(args, arg, values)? {
        return Ok(true);
    }
    if focused::apply_focused_retest_arg(args, arg, values)? {
        return Ok(true);
    }

    Ok(false)
}
