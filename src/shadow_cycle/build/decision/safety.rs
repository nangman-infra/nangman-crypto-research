use crate::model::ShadowCycleDecisionSafety;

pub(super) fn local_shadow_cycle_decision_safety() -> ShadowCycleDecisionSafety {
    ShadowCycleDecisionSafety {
        s3_write: false,
        ecs_task_started: false,
        dispatcher_mode_changed: false,
        local_decision_only: true,
        shadow_status_mutated: false,
        paper_live_enabled: false,
        live_enabled: false,
        order_execution_enabled: false,
    }
}
