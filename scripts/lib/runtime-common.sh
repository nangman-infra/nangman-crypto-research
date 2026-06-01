#!/usr/bin/env bash

RUNTIME_COMMON_LIB_DIR="${RUNTIME_COMMON_LIB_DIR:-$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)}"

# shellcheck source=scripts/lib/runtime-common-core.sh
source "$RUNTIME_COMMON_LIB_DIR/runtime-common-core.sh"
# shellcheck source=scripts/lib/runtime-common-aws.sh
source "$RUNTIME_COMMON_LIB_DIR/runtime-common-aws.sh"
# shellcheck source=scripts/lib/runtime-common-s3.sh
source "$RUNTIME_COMMON_LIB_DIR/runtime-common-s3.sh"
