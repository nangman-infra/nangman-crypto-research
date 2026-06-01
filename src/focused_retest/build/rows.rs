use super::super::types::FocusedRetestRow;
use crate::error::AppResult;
use serde_json::Value;

mod candidate;
mod fields;
mod filters;
mod sort;

use candidate::candidate_rows;
use filters::FocusRowFilters;
pub(super) use sort::horizon_order;
use sort::sort_rows;

pub(super) fn focus_rows(
    status: &Value,
    actions: &[String],
    candidate_lifecycle_key_filter: &[String],
) -> AppResult<Vec<FocusedRetestRow>> {
    let filters = FocusRowFilters::new(actions, candidate_lifecycle_key_filter);
    let mut rows = Vec::new();
    let Some(symbols) = status.get("by_symbol").and_then(Value::as_array) else {
        return Ok(rows);
    };
    for symbol_doc in symbols {
        let symbol = fields::string_field(symbol_doc, "symbol").unwrap_or("UNKNOWN");
        let Some(candidates) = symbol_doc.get("candidates").and_then(Value::as_array) else {
            continue;
        };
        for candidate in candidates {
            rows.extend(candidate_rows(symbol, candidate, &filters));
        }
    }
    sort_rows(&mut rows);
    Ok(rows)
}
