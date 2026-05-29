use crate::alert::config::{AlertConfig, AlertPriority};

use super::metrics::ResearchAlertMetrics;

pub(super) struct ResearchAlertContext {
    pub(super) priority: AlertPriority,
    pub(super) title: String,
    pub(super) conclusion: String,
}

pub(super) fn research_alert_context(
    metrics: &ResearchAlertMetrics,
    config: &AlertConfig,
) -> Option<ResearchAlertContext> {
    if metrics.max_total_notional_pct > 0.0 {
        Some(ResearchAlertContext {
            priority: AlertPriority::P0,
            title: "portfolio notional is non-zero".to_owned(),
            conclusion: "portfolio allocation notional이 0보다 큽니다. live/order 경계가 의도대로 잠겨 있는지 확인해야 합니다."
                .to_owned(),
        })
    } else if metrics.paper_count > 0 || metrics.promote_paper_count > 0 {
        Some(ResearchAlertContext {
            priority: AlertPriority::P1,
            title: "PROMOTE_TO_PAPER 후보 발생".to_owned(),
            conclusion: "shadow를 통과한 paper 후보가 생성됐습니다. 아직 EXECUTION_APPROVED/LIVE_READY는 아닙니다."
                .to_owned(),
        })
    } else if metrics.paper_watch_count > 0 {
        Some(ResearchAlertContext {
            priority: AlertPriority::P2,
            title: format!("모의 관찰 후보 {}개 발생", metrics.paper_watch_count),
            conclusion: "실제 거래가 아니라, 수익 가능성이 보인 RETEST 후보를 돈 안 쓰는 실시간 모의 관찰 단계로 올렸습니다."
                .to_owned(),
        })
    } else if metrics.shadow_count > 0 || metrics.promote_shadow_count > 0 {
        Some(ResearchAlertContext {
            priority: AlertPriority::P2,
            title: "PROMOTE_TO_SHADOW 후보 발생".to_owned(),
            conclusion: "후보가 shadow 관측 단계로 올라갔습니다. 주문 실행은 없고 forward observation만 시작됩니다."
                .to_owned(),
        })
    } else if config.include_retest_summary && metrics.retest_count > 0 {
        Some(ResearchAlertContext {
            priority: AlertPriority::P3,
            title: "RETEST 블로커 요약".to_owned(),
            conclusion: "후보가 아직 PROMOTE로 올라가지 못하고 RETEST 상태입니다.".to_owned(),
        })
    } else {
        None
    }
}
