#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)
PROJECT_ROOT=$(cd "$SCRIPT_DIR/../.." && pwd -P)
CONFIG="$PROJECT_ROOT/.infra/ci/tools.toml"
INSTALL_ROOT="$PROJECT_ROOT/.infra/tools/bin"
BIN_DIR="$INSTALL_ROOT/bin"

die() {
    echo "error: $*" >&2
    exit 1
}

[ "$#" -ge 1 ] || die "usage: infra-tool.sh TOOL [ARGS...]"
TOOL="$1"
shift
[ -f "$CONFIG" ] || die "tool configuration was not found at '$CONFIG'"

tool_value() {
    local key="$1"
    awk -v section="[$TOOL]" -v key="$key" '
        $0 == section { in_section = 1; next }
        /^\[/ { in_section = 0 }
        in_section && $0 ~ "^[[:space:]]*" key "[[:space:]]*=" {
            value = $0
            sub(/^[^=]*=[[:space:]]*"/, "", value)
            sub(/"[[:space:]]*$/, "", value)
            print value
            exit
        }
    ' "$CONFIG"
}

SOURCE=$(tool_value source)
REVISION=$(tool_value revision)
BINARY=$(tool_value binary)
PACKAGE=$(tool_value package)
[ -n "$SOURCE" ] || die "source is missing for '$TOOL'"
[ -n "$REVISION" ] || die "revision is missing for '$TOOL'"
[ -n "$BINARY" ] || die "binary is missing for '$TOOL'"
[ -n "$PACKAGE" ] || die "package is missing for '$TOOL'"

mkdir -p "$BIN_DIR"
TARGET="$BIN_DIR/$BINARY"
if [ ! -x "$TARGET" ]; then
    echo "==> installing $TOOL@$REVISION"
    cargo install --git "$SOURCE" --rev "$REVISION" --locked --root "$INSTALL_ROOT" \
        "$PACKAGE" --bin "$BINARY"
fi

if [ "$TOOL" = "rs-infra-ci" ]; then
    "$SCRIPT_DIR/infra-tool.sh" rs-infra-style --help >/dev/null
    "$SCRIPT_DIR/infra-tool.sh" rs-infra-verify --help >/dev/null
fi

exec env RS_INFRA_BIN_DIR="$BIN_DIR" "$TARGET" "$@"
