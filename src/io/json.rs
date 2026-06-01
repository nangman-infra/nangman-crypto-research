mod filter;
mod input;
mod path;
mod plain;

pub(super) use filter::read_json_array_or_jsonl_bytes_filter;
pub(super) use path::read_json_array_or_jsonl;
pub(super) use plain::read_json_array_or_jsonl_bytes;
