mod artifacts;
mod shared;
mod watch;

pub use artifacts::{PaperArtifacts, build_paper_artifacts, default_paper_account_profile};
pub use shared::is_completed_passed_shadow;
pub use watch::build_paper_watch_candidates;
