use super::*;
use crate::model::ShadowCycleSchedulerAction;
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};

#[path = "cli_tests/support.rs"]
mod support;
use support::*;
#[path = "cli_tests/historical_manifest.rs"]
mod historical_manifest;
#[path = "cli_tests/paper_watch.rs"]
mod paper_watch;
#[path = "cli_tests/research_run.rs"]
mod research_run;
#[path = "cli_tests/retest_horizon.rs"]
mod retest_horizon;
#[path = "cli_tests/retest_scheduler.rs"]
mod retest_scheduler;
#[path = "cli_tests/shadow_decision.rs"]
mod shadow_decision;
