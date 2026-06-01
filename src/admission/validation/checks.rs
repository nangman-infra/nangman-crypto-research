pub(super) fn require(condition: bool, reason: &'static str, reasons: &mut Vec<String>) {
    if !condition {
        reasons.push(reason.to_owned());
    }
}
