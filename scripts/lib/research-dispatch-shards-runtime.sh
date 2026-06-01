# shellcheck shell=bash

RESEARCH_DISPATCH_RUNTIME_LIB_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"

# shellcheck source=scripts/lib/research-dispatch-shards-core.sh
source "$RESEARCH_DISPATCH_RUNTIME_LIB_DIR/research-dispatch-shards-core.sh"
# shellcheck source=scripts/lib/research-dispatch-shards-manifest.sh
source "$RESEARCH_DISPATCH_RUNTIME_LIB_DIR/research-dispatch-shards-manifest.sh"
# shellcheck source=scripts/lib/research-dispatch-shards-aws.sh
source "$RESEARCH_DISPATCH_RUNTIME_LIB_DIR/research-dispatch-shards-aws.sh"
# shellcheck source=scripts/lib/research-dispatch-shards-tasks.sh
source "$RESEARCH_DISPATCH_RUNTIME_LIB_DIR/research-dispatch-shards-tasks.sh"
# shellcheck source=scripts/lib/research-dispatch-shards-reports.sh
source "$RESEARCH_DISPATCH_RUNTIME_LIB_DIR/research-dispatch-shards-reports.sh"
