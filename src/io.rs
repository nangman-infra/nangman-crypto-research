mod json;
mod partition;
mod readers;
mod types;
mod writers;

pub use readers::{
    read_candidate_bundles, read_candidate_bundles_from_bytes, read_market_feature_deltas,
    read_market_feature_deltas_from_bytes, read_market_feature_deltas_matching_symbols_from_bytes,
    read_market_live_ticks_from_bytes, read_market_regime_contexts,
    read_market_regime_contexts_from_bytes, read_oss_adapter_runs,
    read_oss_adapter_runs_from_bytes, read_paper_watch_candidates_from_bytes,
    read_paper_watch_live_marks_from_bytes, read_replay_run_index_records,
    read_replay_run_index_records_from_bytes, read_replay_runs, read_replay_runs_from_bytes,
    read_research_input_manifest, read_research_input_manifest_from_bytes,
    read_research_run_report, read_research_run_report_from_bytes, read_shadow_validation_runs,
    read_shadow_validation_runs_from_bytes,
};
pub use types::{PortfolioOutputBodies, ResearchOutputArtifacts};
pub use writers::{
    write_paper_watch_live_marks, write_portfolio_outputs_to_body, write_pretty_json_file,
    write_research_input_manifest, write_research_outputs, write_shadow_cycle_decision,
    write_shadow_cycle_decision_to_dir,
};
