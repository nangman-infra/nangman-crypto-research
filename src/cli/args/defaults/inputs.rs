use super::*;

pub(super) fn apply_research_input_env(args: &mut Args) {
    args.input_manifest_s3_bucket = env_string("RESEARCH_INPUT_MANIFEST_S3_BUCKET");
    args.input_manifest_s3_key = env_string("RESEARCH_INPUT_MANIFEST_S3_KEY");
    args.research_report_file = env_string("RESEARCH_REPORT_FILE").map(PathBuf::from);
    args.research_report_s3_bucket = env_string("RESEARCH_REPORT_S3_BUCKET");
    args.research_report_s3_key = env_string("RESEARCH_REPORT_S3_KEY");
    args.input_bundle_s3_bucket = env_string("RESEARCH_INPUT_S3_BUCKET");
    args.input_bundle_s3_key = env_string("RESEARCH_INPUT_S3_KEY");
    args.market_l1_s3_bucket = env_string("RESEARCH_MARKET_L1_S3_BUCKET");
    args.market_feature_delta_s3_keys = env_list("RESEARCH_MARKET_FEATURE_DELTA_S3_KEYS");
    args.market_regime_context_s3_keys = env_list("RESEARCH_MARKET_REGIME_CONTEXT_S3_KEYS");
    args.oss_adapter_run_s3_bucket = env_string("RESEARCH_OSS_ADAPTER_RUN_S3_BUCKET");
    args.oss_adapter_run_s3_keys = env_list("RESEARCH_OSS_ADAPTER_RUN_S3_KEYS");
    args.shadow_validation_run_s3_bucket = env_string("RESEARCH_SHADOW_VALIDATION_RUN_S3_BUCKET");
    args.shadow_validation_run_s3_keys = env_list("RESEARCH_SHADOW_VALIDATION_RUN_S3_KEYS");
}
