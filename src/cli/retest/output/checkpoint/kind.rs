pub(super) enum RetestRefreshCheckpointKind {
    Plan,
    Status,
}

impl RetestRefreshCheckpointKind {
    pub(super) fn local_filename(&self) -> &'static str {
        match self {
            Self::Plan => "retest-horizon-plan.json",
            Self::Status => "retest-horizon-status.json",
        }
    }

    pub(super) fn s3_prefix(&self) -> &'static str {
        match self {
            Self::Plan => "retest-horizon-plan/schema=research_retest_horizon_plan_v1",
            Self::Status => "retest-horizon-status/schema=research_horizon_status_checkpoint_v1",
        }
    }
}
