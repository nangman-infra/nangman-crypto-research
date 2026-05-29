#[cfg(test)]
use super::config::APP_NAME;
use super::config::AlertPriority;

#[derive(Debug, Clone)]
pub struct AlertEvent {
    pub(in crate::alert) priority: AlertPriority,
    pub(in crate::alert) title: String,
    pub(in crate::alert) conclusion: String,
    pub(in crate::alert) current_state: Vec<String>,
    pub(in crate::alert) reasons: Vec<String>,
    pub(in crate::alert) next_actions: Vec<String>,
    pub(in crate::alert) safety: Vec<String>,
}

impl AlertEvent {
    #[cfg(test)]
    pub(in crate::alert) fn text(&self, environment: &str) -> String {
        let mut sections = vec![
            format!("[{}][{}] {}", self.priority.as_str(), APP_NAME, self.title),
            String::new(),
            "결론:".to_owned(),
            self.conclusion.clone(),
            String::new(),
            "현재 상태:".to_owned(),
        ];
        sections.extend(bullet_lines(&self.current_state));
        sections.push(format!("- env: {environment}"));
        append_section(&mut sections, "주요 원인:", &self.reasons);
        append_section(&mut sections, "다음 행동:", &self.next_actions);
        append_section(&mut sections, "안전 상태:", &self.safety);
        sections.join("\n")
    }
}

#[cfg(test)]
fn bullet_lines(values: &[String]) -> Vec<String> {
    values.iter().map(|value| format!("- {value}")).collect()
}

#[cfg(test)]
fn append_section(sections: &mut Vec<String>, title: &str, values: &[String]) {
    if values.is_empty() {
        return;
    }
    sections.push(String::new());
    sections.push(title.to_owned());
    sections.extend(bullet_lines(values));
}
