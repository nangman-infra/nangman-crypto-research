use super::Args;

mod paper_watch;
mod retest;
mod shadow;

pub(super) use paper_watch::*;
pub(super) use retest::*;
pub(super) use shadow::*;

pub(super) fn has_retest_horizon_status_input(args: &Args) -> bool {
    args.retest_horizon_status_file.is_some() || args.retest_horizon_status_s3_key.is_some()
}

pub(super) fn has_retest_horizon_plan_input(args: &Args) -> bool {
    args.retest_horizon_plan_file.is_some() || args.retest_horizon_plan_s3_key.is_some()
}

pub(super) fn has_research_report_input(args: &Args) -> bool {
    args.research_report_file.is_some() || args.research_report_s3_key.is_some()
}
