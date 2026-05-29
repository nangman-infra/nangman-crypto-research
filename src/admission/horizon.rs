pub fn horizon_ms(value: &str) -> Option<i64> {
    match value.trim() {
        "15m" => Some(15 * 60 * 1000),
        "1h" => Some(60 * 60 * 1000),
        "4h" => Some(4 * 60 * 60 * 1000),
        "24h" => Some(24 * 60 * 60 * 1000),
        "72h" => Some(72 * 60 * 60 * 1000),
        "7d" => Some(7 * 24 * 60 * 60 * 1000),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::horizon_ms;

    #[test]
    fn horizon_ms_resolves_supported_replay_windows() {
        assert_eq!(horizon_ms("15m"), Some(15 * 60 * 1000));
        assert_eq!(horizon_ms("1h"), Some(60 * 60 * 1000));
        assert_eq!(horizon_ms("4h"), Some(4 * 60 * 60 * 1000));
        assert_eq!(horizon_ms("24h"), Some(24 * 60 * 60 * 1000));
        assert_eq!(horizon_ms("72h"), Some(72 * 60 * 60 * 1000));
        assert_eq!(horizon_ms("7d"), Some(7 * 24 * 60 * 60 * 1000));
    }

    #[test]
    fn horizon_ms_trims_input_and_rejects_unknown_windows() {
        assert_eq!(horizon_ms(" 1h "), Some(60 * 60 * 1000));
        assert_eq!(horizon_ms("2h"), None);
    }
}
