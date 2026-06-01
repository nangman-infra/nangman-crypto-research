use super::*;

#[tokio::test]
async fn report_id_and_output_key_are_stable_without_now_ms() {
    let root = test_root("stable-report");
    let input = root.join("bundles.jsonl");
    let output_a = root.join("out-a");
    let output_b = root.join("out-b");
    write_json(&input, &bundle_json());

    let args = |output_dir: PathBuf| Args {
        input_bundle_file: Some(input.clone()),
        output_dir: Some(output_dir),
        research_packet_id: "packet_test".to_owned(),
        run_scope: "test".to_owned(),
        now_ms: None,
        ..default_args()
    };

    let summary_a = run(args(output_a.clone()))
        .await
        .expect("first run succeeds");
    let summary_b = run(args(output_b.clone()))
        .await
        .expect("second run succeeds");
    let report_a: Value = serde_json::from_str(
        &fs::read_to_string(&summary_a.output_files[0]).expect("first report exists"),
    )
    .expect("first report json parses");
    let report_b: Value = serde_json::from_str(
        &fs::read_to_string(&summary_b.output_files[0]).expect("second report exists"),
    )
    .expect("second report json parses");

    assert_eq!(
        report_a["research_run_report_id"],
        report_b["research_run_report_id"]
    );
    assert_eq!(report_a["created_at_ms"], json!(7_200_000));
    assert_eq!(report_b["created_at_ms"], json!(7_200_000));
    let relative_a = Path::new(&summary_a.output_files[0])
        .strip_prefix(&output_a)
        .expect("first output is under output dir");
    let relative_b = Path::new(&summary_b.output_files[0])
        .strip_prefix(&output_b)
        .expect("second output is under output dir");
    assert_eq!(relative_a, relative_b);
}
