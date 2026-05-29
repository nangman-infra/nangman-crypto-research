mod candidates;
mod nats;
mod output;
mod restore;

use super::super::*;

use self::candidates::load_paper_watch_observer_candidates;
pub(in crate::cli) use self::nats::paper_watch_observer_nats_config;
pub(in crate::cli) use self::output::{
    write_paper_watch_observer_live_marks, write_paper_watch_observer_snapshot,
};
use self::restore::restore_paper_watch_observer_state;

pub(in crate::cli) async fn run_paper_watch_observer_mode(args: &Args) -> AppResult<RunSummary> {
    let observer_run_id = format!(
        "paper_watch_observer_{}",
        args.now_ms.unwrap_or_else(now_ms)
    );
    let mut state = PaperWatchObserverState::default();
    let restored_mark_count = restore_paper_watch_observer_state(args, &mut state).await?;
    let mut iteration = 0usize;
    let mut total_new_marks = 0usize;
    let mut snapshots_created = 0usize;
    let mut latest_active_candidates: usize;
    let mut output_files = Vec::new();

    loop {
        iteration += 1;
        let iteration_now_ms = args.now_ms.unwrap_or_else(now_ms);
        let candidates = load_paper_watch_observer_candidates(args).await?;
        let active = active_candidates(&candidates, iteration_now_ms);
        latest_active_candidates = active.len();
        let ticks = if active.is_empty() {
            Vec::new()
        } else {
            read_market_live_ticks_from_nats(&paper_watch_observer_nats_config(args)?).await?
        };
        let new_marks = state.ingest_ticks(&active, &ticks);
        total_new_marks += new_marks.len();
        output_files.extend(
            write_paper_watch_observer_live_marks(args, &new_marks, iteration_now_ms).await?,
        );

        let snapshot = state.snapshot(
            &observer_run_id,
            iteration,
            iteration_now_ms,
            &candidates,
            &new_marks,
        );
        output_files
            .push(write_paper_watch_observer_snapshot(args, &snapshot, iteration_now_ms).await?);
        snapshots_created += 1;

        if args.paper_watch_observer_max_iterations > 0
            && iteration >= args.paper_watch_observer_max_iterations
        {
            break;
        }
        sleep(Duration::from_secs(args.paper_watch_observer_poll_secs)).await;
    }

    Ok(RunSummary {
        paper_watch_live_marks_created: total_new_marks,
        paper_watch_observer_iterations: iteration,
        paper_watch_observer_snapshots_created: snapshots_created,
        paper_watch_observer_active_candidates: latest_active_candidates,
        paper_watch_observer_restored_live_marks: restored_mark_count,
        output_files,
        ..RunSummary::default()
    })
}
