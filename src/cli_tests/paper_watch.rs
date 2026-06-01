use super::*;
use serde_json::{Value, json};
use std::fs;

#[path = "paper_watch/candidate.rs"]
mod candidate;
#[path = "paper_watch/live_cycle.rs"]
mod live_cycle;
#[path = "paper_watch/observer.rs"]
mod observer;
#[path = "paper_watch/validation.rs"]
mod validation;
