mod state;
mod stats;
mod summary;
#[cfg(test)]
mod tests;
mod types;
mod verdict;

pub use self::state::PaperWatchObserverState;
pub use self::summary::active_candidates;
pub use self::types::{
    PAPER_WATCH_OBSERVER_SNAPSHOT_SCHEMA_VERSION, PaperWatchObserverCandidateSummary,
    PaperWatchObserverReturnSummary, PaperWatchObserverSafety, PaperWatchObserverSnapshot,
};
