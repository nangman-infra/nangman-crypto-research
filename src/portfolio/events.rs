use crate::hash::stable_id;
use crate::model::{
    PORTFOLIO_REDUCE_ONLY_SIGNAL_SCHEMA_VERSION, PORTFOLIO_RISK_REJECT_EVENT_SCHEMA_VERSION,
    PortfolioReduceOnlySignal, PortfolioRiskRejectEvent,
};

pub(super) fn reject_event(
    candidate_lifecycle_key: &str,
    symbol: &str,
    reason: &str,
    computed_at_ms: i64,
) -> PortfolioRiskRejectEvent {
    PortfolioRiskRejectEvent {
        portfolio_risk_reject_event_id: stable_id(
            "portfolio_risk_reject",
            &[
                candidate_lifecycle_key,
                symbol,
                reason,
                &computed_at_ms.to_string(),
            ],
        ),
        schema_version: PORTFOLIO_RISK_REJECT_EVENT_SCHEMA_VERSION.to_owned(),
        candidate_lifecycle_key: candidate_lifecycle_key.to_owned(),
        symbol_canonical: symbol.to_owned(),
        reason: reason.to_owned(),
        computed_at_ms,
    }
}

pub(super) fn reduce_only_signal(
    symbol: &str,
    reason: &str,
    computed_at_ms: i64,
) -> PortfolioReduceOnlySignal {
    PortfolioReduceOnlySignal {
        portfolio_reduce_only_signal_id: stable_id(
            "portfolio_reduce_only",
            &[symbol, reason, &computed_at_ms.to_string()],
        ),
        schema_version: PORTFOLIO_REDUCE_ONLY_SIGNAL_SCHEMA_VERSION.to_owned(),
        symbol_canonical: symbol.to_owned(),
        reason: reason.to_owned(),
        computed_at_ms,
    }
}
