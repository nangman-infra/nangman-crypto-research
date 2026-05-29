mod live_cycle;
mod observer;
mod sources;

pub(super) use live_cycle::run_paper_watch_live_cycle_mode;
pub(super) use observer::run_paper_watch_observer_mode;

#[cfg(test)]
pub(super) use observer::{
    paper_watch_observer_nats_config, write_paper_watch_observer_live_marks,
    write_paper_watch_observer_snapshot,
};
#[cfg(test)]
pub(super) use sources::market_live_nats_configs_for_candidates;
