use super::{Args, ResearchInputs};

pub(super) fn resolve_research_identity<'a>(
    args: &'a Args,
    inputs: &'a ResearchInputs,
) -> (&'a str, &'a str) {
    let research_packet_id = inputs
        .manifest
        .as_ref()
        .and_then(|manifest| manifest.research_packet_id.as_deref())
        .unwrap_or(&args.research_packet_id);
    let run_scope = inputs
        .manifest
        .as_ref()
        .and_then(|manifest| manifest.run_scope.as_deref())
        .unwrap_or(&args.run_scope);
    (research_packet_id, run_scope)
}
