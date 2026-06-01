use super::{
    MS_PER_HOUR, build_shadow_cycle_decision, shadow_sample_deficit_lifecycle_keys,
    validate_shadow_cycle_decision,
};
use crate::model::{
    HOLDING_POLICY_VERSION, HoldingPolicy, ShadowCycleDecision, ShadowCycleSchedulerAction,
    ShadowStartConditionSummary, ShadowTerminationPolicy, ShadowValidationRun,
    ShadowValidationStatus, ShadowWatchWindowPolicy, SurvivalBand,
};

#[path = "tests/contracts.rs"]
mod contracts;
#[path = "tests/decision_build.rs"]
mod decision_build;
#[path = "tests/fixtures.rs"]
mod fixtures;

use fixtures::*;
