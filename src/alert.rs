mod builders;
mod config;
mod delivery;
mod event;

use builders::{
    paper_watch_live_mark_alert_event, research_report_alert_event,
    shadow_cycle_decision_alert_event,
};
use config::AlertConfig;
pub use config::AlertPriority;
use delivery::send_event;
pub use event::AlertEvent;

use crate::model::{
    PaperWatchCandidate, PaperWatchLiveMark, ResearchRunReport, ShadowCycleDecision,
};

#[cfg(test)]
use crate::model::{ResearchBias, ShadowCycleSchedulerAction};
#[cfg(test)]
use builders::top_reason_lines;
#[cfg(test)]
use config::{APP_NAME, DEFAULT_PIPELINE_ALERT_S3_PREFIX};
#[cfg(test)]
use delivery::{
    PipelineAlertEvent, build_pipeline_alert_delivery, pipeline_alert_event_key, s3_key_token,
};

pub async fn emit_research_report_alert_from_env(
    report: &ResearchRunReport,
    paper_watch_candidates: &[PaperWatchCandidate],
) {
    let Some(config) = AlertConfig::from_env() else {
        return;
    };
    let Some(event) = research_report_alert_event(report, paper_watch_candidates, &config) else {
        return;
    };
    if let Err(error) = send_event(&config, &event).await {
        eprintln!("pipeline alert delivery failed: {error}");
    }
}

pub async fn emit_paper_watch_live_mark_alert_from_env(marks: &[PaperWatchLiveMark]) {
    let Some(config) = AlertConfig::from_env() else {
        return;
    };
    let Some(event) = paper_watch_live_mark_alert_event(marks, &config) else {
        return;
    };
    if let Err(error) = send_event(&config, &event).await {
        eprintln!("pipeline alert delivery failed: {error}");
    }
}

pub async fn emit_shadow_cycle_decision_alert_from_env(decision: &ShadowCycleDecision) {
    let Some(config) = AlertConfig::from_env() else {
        return;
    };
    let Some(event) = shadow_cycle_decision_alert_event(decision, &config) else {
        return;
    };
    if let Err(error) = send_event(&config, &event).await {
        eprintln!("pipeline alert delivery failed: {error}");
    }
}

#[cfg(test)]
mod tests;
