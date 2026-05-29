use crate::error::AppResult;

use super::super::super::partition::partition;

pub(super) struct ResearchOutputKeys {
    date: String,
    hour: u32,
    report_id: String,
}

impl ResearchOutputKeys {
    pub(super) fn new(timestamp_ms: i64, report_id: &str) -> AppResult<Self> {
        let dt = partition(timestamp_ms)?;
        Ok(Self {
            date: dt.date,
            hour: dt.hour,
            report_id: report_id.to_owned(),
        })
    }

    pub(super) fn jsonl_dataset(&self, dataset: &str, schema_version: &str) -> String {
        self.object_path(dataset, schema_version, "part-000001.jsonl")
    }

    pub(super) fn json_object(
        &self,
        dataset: &str,
        schema_version: &str,
        file_name: &str,
    ) -> String {
        self.object_path(dataset, schema_version, file_name)
    }

    fn object_path(&self, dataset: &str, schema_version: &str, file_name: &str) -> String {
        format!(
            "{dataset}/schema={schema_version}/dt={}/hour={:02}/research_run_report_id={}/{file_name}",
            self.date, self.hour, self.report_id
        )
    }
}
