use research_app::cli::{RunSummary, parse_args, print_help, run};
use std::env;
use std::process;

#[tokio::main]
async fn main() {
    let result = match parse_args(env::args().skip(1)) {
        Ok(Some(args)) => run(args).await,
        Ok(None) => {
            print_help();
            Ok(RunSummary {
                retest_horizon_statuses_validated: 0,
                retest_cycle_scheduler_action: None,
                retest_cycle_run_not_before_ms: None,
                shadow_cycle_decisions_validated: 0,
                shadow_cycle_decisions_created: 0,
                shadow_cycle_scheduler_action: None,
                shadow_cycle_run_not_before_ms: None,
                shadow_cycle_focused_research_manifest_file: None,
                processed_bundles: 0,
                replay_runs_created: 0,
                historical_replay_runs_loaded: 0,
                oss_adapter_runs_loaded: 0,
                shadow_validation_runs_loaded: 0,
                shadow_validation_runs_created: 0,
                paper_trade_candidates_created: 0,
                paper_trade_runs_created: 0,
                paper_trade_summaries_created: 0,
                paper_trade_marks_created: 0,
                portfolio_risk_reject_events_created: 0,
                portfolio_reduce_only_signals_created: 0,
                output_files: Vec::new(),
            })
        }
        Err(error) => Err(error),
    };

    match result {
        Ok(summary) => {
            if summary.processed_bundles > 0
                || summary.retest_horizon_statuses_validated > 0
                || summary.shadow_cycle_decisions_validated > 0
                || summary.shadow_cycle_decisions_created > 0
            {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&summary).unwrap_or_default()
                );
            }
        }
        Err(error) => {
            eprintln!("{error}");
            process::exit(1);
        }
    }
}
