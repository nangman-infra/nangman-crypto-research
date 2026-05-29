#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::storage) struct ListedPayloadObject {
    pub(super) key: String,
    pub(super) last_modified_ms: i64,
}

pub(in crate::storage) fn select_latest_payload_keys(
    mut objects: Vec<ListedPayloadObject>,
    read_limit: usize,
) -> Vec<String> {
    objects.sort_by(|left, right| {
        right
            .last_modified_ms
            .cmp(&left.last_modified_ms)
            .then_with(|| right.key.cmp(&left.key))
    });
    objects
        .into_iter()
        .take(read_limit)
        .map(|object| object.key)
        .collect()
}
