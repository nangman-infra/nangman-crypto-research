const MS_PER_HOUR: i64 = 60 * 60 * 1000;

mod build;
mod io;
mod validation;

pub use build::{build_shadow_cycle_decision, shadow_sample_deficit_lifecycle_keys};
pub use io::read_shadow_cycle_decision;
pub use validation::validate_shadow_cycle_decision;

#[cfg(test)]
mod tests;
