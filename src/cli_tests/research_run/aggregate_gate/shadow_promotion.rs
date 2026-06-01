use super::*;

#[path = "shadow_promotion/assertions.rs"]
mod assertions;
#[path = "shadow_promotion/setup.rs"]
mod setup;

use assertions::{assert_registry, assert_report, assert_shadow_output};
use setup::{shadow_promotion_args, write_shadow_promotion_inputs};

#[tokio::test]
async fn aggregate_gate_promotes_only_to_shadow_when_enterprise_blockers_clear() {
    let root = test_root("aggregate-shadow");
    let input_paths = write_shadow_promotion_inputs(&root);

    let summary = run(shadow_promotion_args(input_paths))
        .await
        .expect("run succeeds");

    assert_shadow_output(&summary);
    assert_report(&summary);
    assert_registry(&summary);
}
