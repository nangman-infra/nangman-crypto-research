use super::*;

#[test]
fn pipeline_alert_event_key_is_hour_partitioned() {
    let key = pipeline_alert_event_key(
        "pipeline-alert-event/schema=pipeline_alert_event_v1/",
        1779937200123,
        "research-app",
        "P2",
        "pipeline_alert_abc123",
    )
    .expect("timestamp is valid");

    assert_eq!(
        key,
        "pipeline-alert-event/schema=pipeline_alert_event_v1/dt=2026-05-28/hour=03/app=research-app/priority=P2/pipeline_alert_abc123.json"
    );
}

#[test]
fn pipeline_alert_event_payload_preserves_operator_sections() {
    let event = AlertEvent {
        priority: AlertPriority::P2,
        title: "모의 관찰 후보 2개 발생".to_owned(),
        conclusion: "paper-watch 후보를 관찰 단계로 올렸습니다.".to_owned(),
        current_state: vec!["관찰 코인: DOGE, XRP".to_owned()],
        reasons: vec!["과거 검증은 긍정적이지만 승급 조건이 아직 부족함: 2개".to_owned()],
        next_actions: vec!["실제 주문 없이 live mark를 계속 누적합니다.".to_owned()],
        safety: vec!["실제 주문: 꺼짐".to_owned()],
    };

    let payload = PipelineAlertEvent::from_alert_event(
        &event,
        "pipeline_alert_test",
        "pipeline_alert_dedupe_test",
        "dev",
        1779937200123,
    );
    let json = serde_json::to_value(&payload).expect("payload serializes");

    assert_eq!(json["schema_version"], "pipeline_alert_event_v1");
    assert_eq!(json["app"], APP_NAME);
    assert_eq!(json["priority"], "P2");
    assert_eq!(json["title"], "모의 관찰 후보 2개 발생");
    assert_eq!(json["current_state"][0], "관찰 코인: DOGE, XRP");
    assert_eq!(json["safety"][0], "실제 주문: 꺼짐");
    assert_eq!(json["created_at_ms"], 1779937200123_i64);
}

#[test]
fn build_pipeline_alert_delivery_writes_expected_key_and_body() {
    let config = test_config(AlertPriority::P2);
    let event = AlertEvent {
        priority: AlertPriority::P2,
        title: "PROMOTE_TO_SHADOW 후보 발생".to_owned(),
        conclusion: "후보가 shadow 관측 단계로 올라갔습니다.".to_owned(),
        current_state: vec!["shadow 관찰 생성: 1개".to_owned()],
        reasons: vec!["deterministic_shadow_gate_passed: 1개".to_owned()],
        next_actions: vec!["주문 실행은 계속 꺼둡니다.".to_owned()],
        safety: vec!["실제 주문: 꺼짐".to_owned()],
    };

    let delivery =
        build_pipeline_alert_delivery(&config, &event, 1779937200123).expect("delivery is built");
    let payload: serde_json::Value =
        serde_json::from_slice(&delivery.body).expect("body is valid json");

    assert!(delivery.key.starts_with(
            "pipeline-alert-event/schema=pipeline_alert_event_v1/dt=2026-05-28/hour=03/app=research-app/priority=P2/"
        ));
    assert_eq!(payload["schema_version"], "pipeline_alert_event_v1");
    assert_eq!(payload["environment"], "dev");
    assert_eq!(payload["title"], "PROMOTE_TO_SHADOW 후보 발생");
    assert_eq!(payload["next_actions"][0], "주문 실행은 계속 꺼둡니다.");
    assert!(
        payload["event_id"]
            .as_str()
            .is_some_and(|value| value.starts_with("pipeline_alert_"))
    );
    assert!(
        payload["dedupe_key"]
            .as_str()
            .is_some_and(|value| value.starts_with("pipeline_alert_dedupe_"))
    );
}

#[test]
fn s3_key_token_replaces_unsafe_characters() {
    assert_eq!(s3_key_token("research app/P2"), "research_app_P2");
    assert_eq!(
        s3_key_token("pipeline_alert_abc-123"),
        "pipeline_alert_abc-123"
    );
}

#[test]
fn pipeline_alert_event_key_rejects_invalid_timestamp() {
    let error = pipeline_alert_event_key(
        DEFAULT_PIPELINE_ALERT_S3_PREFIX,
        i64::MAX,
        "research-app",
        "P2",
        "event",
    )
    .expect_err("invalid timestamp is rejected");

    assert_eq!(error, "created_at_ms is outside supported timestamp range");
}
