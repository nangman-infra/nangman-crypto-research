use super::*;

pub(in crate::cli::tests) const DAY_MS: i64 = 24 * 60 * 60 * 1000;

pub(in crate::cli::tests) fn test_root(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "research-app-{name}-{}-{}",
        std::process::id(),
        now_ms()
    ))
}

pub(in crate::cli::tests) fn write_json(path: &Path, value: &Value) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("test parent directory is created");
    }
    fs::write(
        path,
        serde_json::to_vec_pretty(value).expect("test json serializes"),
    )
    .expect("test json is written");
}

pub(in crate::cli::tests) fn output_file_containing(summary: &RunSummary, needle: &str) -> PathBuf {
    summary
        .output_files
        .iter()
        .find(|path| path.contains(needle))
        .map(PathBuf::from)
        .unwrap_or_else(|| panic!("expected output file containing {needle}"))
}
