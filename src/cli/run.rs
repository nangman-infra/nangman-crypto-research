use super::*;

mod direct_mode;
mod modes;
mod outputs;
mod pipeline;
mod research_inputs;

use modes::run_requested_mode;
use pipeline::run_research_pipeline;

pub async fn run(args: Args) -> AppResult<RunSummary> {
    if let Some(summary) = run_requested_mode(&args).await? {
        return Ok(summary);
    }
    run_research_pipeline(&args).await
}
