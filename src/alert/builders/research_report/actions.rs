use crate::alert::config::AlertPriority;

pub(super) fn research_next_actions(priority: AlertPriority) -> Vec<String> {
    match priority {
        AlertPriority::P0 => vec![
            "live/order 설정이 의도적으로 열린 것인지 즉시 확인합니다.".to_owned(),
            "의도한 변경이 아니면 승급 흐름을 멈춥니다.".to_owned(),
        ],
        AlertPriority::P1 => vec![
            "paper 후보 계약을 먼저 검토하고, 실행 경계 변경은 별도로 승인합니다.".to_owned(),
            "실제 주문과 live trading은 계속 꺼둡니다.".to_owned(),
        ],
        AlertPriority::P2 => vec![
            "실제 주문 없이 paper-watch live mark를 계속 누적합니다.".to_owned(),
            "관찰이 끝나기 전에는 live 승인으로 올리지 않습니다.".to_owned(),
        ],
        AlertPriority::P3 => vec![
            "부족한 샘플과 시장 데이터 구간을 focused retest로 채웁니다.".to_owned(),
            "샘플, unseen, train/validation, liquidity 증거가 풀릴 때까지 RETEST로 둡니다."
                .to_owned(),
        ],
    }
}
