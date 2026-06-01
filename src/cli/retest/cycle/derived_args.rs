use super::*;

pub(super) fn derive_latest_state_refresh_args(args: &Args, state: RetestCycleSourceState) -> Args {
    let mut derived_args = args.clone();
    derived_args.run_retest_refresh_cycle_from_latest_state = false;
    derived_args.run_retest_refresh_cycle = true;
    derived_args.input_manifest_file = None;
    derived_args.input_manifest_s3_bucket = Some(state.source_manifest_s3_bucket);
    derived_args.input_manifest_s3_key = Some(state.source_manifest_s3_key);
    derived_args.research_report_file = None;
    derived_args.research_report_s3_bucket = Some(state.source_research_report_s3_bucket);
    derived_args.research_report_s3_key = Some(state.source_research_report_s3_key);
    derived_args
}
