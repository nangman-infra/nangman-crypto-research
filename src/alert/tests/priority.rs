use super::*;

#[test]
fn parses_priority_and_filters_by_minimum_priority() {
    assert_eq!(AlertPriority::parse("p0"), Some(AlertPriority::P0));
    assert_eq!(AlertPriority::parse(" P3 "), Some(AlertPriority::P3));
    assert_eq!(AlertPriority::parse("later"), None);

    let config = test_config(AlertPriority::P2);
    assert!(config.allows(AlertPriority::P0));
    assert!(config.allows(AlertPriority::P2));
    assert!(!config.allows(AlertPriority::P3));
}
