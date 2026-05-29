use crate::error::AppResult;
use crate::model::{
    PortfolioAllocationSnapshot, PortfolioReduceOnlySignal, PortfolioRiskRejectEvent,
};

use super::super::types::PortfolioOutputBodies;

pub fn write_portfolio_outputs_to_body(
    snapshot: &Option<PortfolioAllocationSnapshot>,
    rejects: &[PortfolioRiskRejectEvent],
    reduce_only_signals: &[PortfolioReduceOnlySignal],
) -> AppResult<PortfolioOutputBodies> {
    let snapshot_body = snapshot
        .as_ref()
        .map(serde_json::to_vec_pretty)
        .transpose()?;
    let mut reject_body = Vec::new();
    for record in rejects {
        serde_json::to_writer(&mut reject_body, record)?;
        reject_body.push(b'\n');
    }
    let mut reduce_only_body = Vec::new();
    for record in reduce_only_signals {
        serde_json::to_writer(&mut reduce_only_body, record)?;
        reduce_only_body.push(b'\n');
    }
    Ok((snapshot_body, reject_body, reduce_only_body))
}
