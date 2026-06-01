use super::input::trimmed_utf8;
use crate::error::{AppError, AppResult};
use serde::Deserialize;
use serde::de::{DeserializeOwned, DeserializeSeed, SeqAccess, Visitor};
use std::marker::PhantomData;

pub(in crate::io) fn read_json_array_or_jsonl_bytes_filter<T, F>(
    label: &str,
    bytes: &[u8],
    mut keep: F,
) -> AppResult<Vec<T>>
where
    T: DeserializeOwned,
    F: FnMut(&T) -> bool,
{
    let trimmed = trimmed_utf8(label, bytes)?;
    if trimmed.starts_with('[') {
        let mut deserializer = serde_json::Deserializer::from_str(trimmed);
        let values = FilteredSeqSeed::<T, F> {
            keep: &mut keep,
            marker: PhantomData,
        }
        .deserialize(&mut deserializer)?;
        deserializer.end()?;
        return Ok(values);
    }
    if trimmed.starts_with('{')
        && let Ok(value) = serde_json::from_str(trimmed)
    {
        return Ok(if keep(&value) {
            vec![value]
        } else {
            Vec::new()
        });
    }

    let mut values = Vec::new();
    for (index, line) in trimmed.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let value = serde_json::from_str(line).map_err(|error| {
            AppError::Json(format!(
                "{label} line {} is not valid JSON: {error}",
                index + 1
            ))
        })?;
        if keep(&value) {
            values.push(value);
        }
    }
    Ok(values)
}

struct FilteredSeqSeed<'a, T, F> {
    keep: &'a mut F,
    marker: PhantomData<T>,
}

impl<'de, T, F> DeserializeSeed<'de> for FilteredSeqSeed<'_, T, F>
where
    T: Deserialize<'de>,
    F: FnMut(&T) -> bool,
{
    type Value = Vec<T>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(FilteredSeqVisitor {
            keep: self.keep,
            marker: PhantomData,
        })
    }
}

struct FilteredSeqVisitor<'a, T, F> {
    keep: &'a mut F,
    marker: PhantomData<T>,
}

impl<'de, T, F> Visitor<'de> for FilteredSeqVisitor<'_, T, F>
where
    T: Deserialize<'de>,
    F: FnMut(&T) -> bool,
{
    type Value = Vec<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a JSON array")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = seq.next_element::<T>()? {
            if (self.keep)(&value) {
                values.push(value);
            }
        }
        Ok(values)
    }
}
