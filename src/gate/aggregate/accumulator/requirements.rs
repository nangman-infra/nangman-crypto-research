use super::state::AggregateAccumulator;
use crate::model::IntelCandidateEvidenceBundle;

impl AggregateAccumulator {
    pub(in crate::gate::aggregate) fn apply_bundle_requirements(
        &mut self,
        bundle: Option<&IntelCandidateEvidenceBundle>,
    ) {
        let Some(bundle) = bundle else {
            return;
        };
        self.required_unseen_windows = self
            .required_unseen_windows
            .max(bundle.validation_requirements.min_unseen_windows);
        self.train_validation_split_required |= bundle
            .validation_requirements
            .required_train_validation_split;
        self.liquidity_filter_required |= bundle.validation_requirements.include_liquidity_filter;
    }
}
