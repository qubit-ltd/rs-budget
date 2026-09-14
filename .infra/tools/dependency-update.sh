#!/bin/bash
################################################################################
#
#    Copyright (c) 2025 - 2026 Haixing Hu.
#
#    SPDX-License-Identifier: Apache-2.0
#
#    Licensed under the Apache License, Version 2.0.
#
################################################################################

set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)
PROJECT_ROOT=$(cd "$SCRIPT_DIR/../.." && pwd -P)
POLICY_CONFIG="$PROJECT_ROOT/.infra/dep/policy.toml"
TOOL_RUNNER="$PROJECT_ROOT/.infra/tools/infra-tool.sh"
MODE="update"

usage() {
    cat <<'EOF_USAGE'
Usage: ./dependency-update.sh [--check|--update]

Check or synchronize the project's direct dependencies with the pinned
dependency baseline in .infra/dep/policy.toml.

Options:
  --check   Check only; do not modify Cargo.toml files.
  --update  Synchronize dependency declarations, then check them (the default).
  -h, --help
EOF_USAGE
}

die() {
    echo "error: $*" >&2
    exit 1
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        --check)
            MODE="check"
            ;;
        --update)
            MODE="update"
            ;;
        -h | --help)
            usage
            exit 0
            ;;
        *)
            die "unknown option '$1'"
            ;;
    esac
    shift
done

[ -f "$PROJECT_ROOT/Cargo.toml" ] || die "Cargo.toml was not found at '$PROJECT_ROOT/Cargo.toml'"
[ -f "$POLICY_CONFIG" ] || die "dependency policy configuration was not found at '$POLICY_CONFIG'"

run_policy() {
    "$TOOL_RUNNER" rs-infra-dependency --project "$PROJECT_ROOT" "$@"
}

cd "$PROJECT_ROOT"

if [ "$MODE" = "check" ]; then
    run_policy check
    echo "Dependency baseline check passed."
    exit 0
fi

echo "==> synchronizing dependency declarations"
run_policy sync
echo "==> checking dependency baseline"
run_policy check
echo "Dependency baseline is synchronized."
