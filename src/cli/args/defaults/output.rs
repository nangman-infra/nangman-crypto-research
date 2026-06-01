use super::*;

pub(super) fn apply_history_and_output_env(args: &mut Args) {
    args.historical_replay_run_s3_bucket = env_string("RESEARCH_HISTORICAL_REPLAY_RUN_S3_BUCKET");
    args.historical_replay_run_s3_keys = env_list("RESEARCH_HISTORICAL_REPLAY_RUN_S3_KEYS");
    args.historical_replay_run_index_s3_bucket =
        env_string("RESEARCH_HISTORICAL_REPLAY_RUN_INDEX_S3_BUCKET");
    args.historical_replay_run_index_s3_keys =
        env_list("RESEARCH_HISTORICAL_REPLAY_RUN_INDEX_S3_KEYS");
    args.output_s3_bucket = env_string("RESEARCH_OUTPUT_S3_BUCKET");
    args.output_s3_prefix = env_string("RESEARCH_OUTPUT_S3_PREFIX");
}
