use crate::alert::config::{AlertConfig, AlertPriority};
use crate::alert::event::AlertEvent;
use crate::model::{ShadowCycleDecision, ShadowCycleSchedulerAction};

pub(in crate::alert) fn shadow_cycle_decision_alert_event(
    decision: &ShadowCycleDecision,
    config: &AlertConfig,
) -> Option<AlertEvent> {
    let priority = if decision.safety.live_enabled || decision.safety.order_execution_enabled {
        AlertPriority::P0
    } else if decision
        .scheduler_action
        .requires_focused_research_manifest()
    {
        AlertPriority::P2
    } else if config.include_shadow_wait && decision.scheduler_action.is_wait_action() {
        AlertPriority::P3
    } else {
        return None;
    };

    if !config.allows(priority) {
        return None;
    }

    let title = match decision.scheduler_action {
        ShadowCycleSchedulerAction::RunFocusedShadowSampleAccumulationResearch => {
            "shadow sample accumulation dispatch 준비".to_owned()
        }
        ShadowCycleSchedulerAction::WaitUntilTargetWindowMaterializes
        | ShadowCycleSchedulerAction::WaitUntilPendingShadowTargetWindowMaterializes => {
            "shadow holding window 대기".to_owned()
        }
        _ if priority == AlertPriority::P0 => "shadow cycle safety boundary changed".to_owned(),
        _ => "shadow cycle decision".to_owned(),
    };

    Some(AlertEvent {
        priority,
        title,
        conclusion: format!(
            "shadow cycle scheduler action은 {:?}입니다.",
            decision.scheduler_action
        ),
        current_state: vec![
            format!("decision_id: {}", decision.decision_id),
            format!("source_verdict: {}", decision.source_verdict),
            format!(
                "run_not_before: {}",
                decision
                    .run_not_before_at
                    .clone()
                    .unwrap_or_else(|| "none".to_owned())
            ),
            format!(
                "shadow_validation_count: {}",
                decision.shadow_sample_state.shadow_validation_count
            ),
            format!(
                "target_window_materialized_count: {}",
                decision
                    .shadow_sample_state
                    .target_window_materialized_count
            ),
            format!(
                "pending_target_window_candidate_count: {}",
                decision
                    .shadow_sample_state
                    .pending_target_window_candidate_count
            ),
            format!(
                "total_sample_deficit: {}",
                decision.shadow_sample_state.total_sample_deficit
            ),
        ],
        reasons: decision.blocked_actions.clone(),
        next_actions: decision.safe_next_actions.clone(),
        safety: vec![
            format!("s3_write: {}", decision.safety.s3_write),
            format!("ecs_task_started: {}", decision.safety.ecs_task_started),
            format!("paper_live_enabled: {}", decision.safety.paper_live_enabled),
            format!("live_enabled: {}", decision.safety.live_enabled),
            format!(
                "order_execution_enabled: {}",
                decision.safety.order_execution_enabled
            ),
        ],
    })
}
