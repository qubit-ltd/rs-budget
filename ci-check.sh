#!/bin/bash
set -euo pipefail

PROJECT_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)
exec "$PROJECT_ROOT/.infra/tools/infra-tool.sh" rs-infra-ci --project "$PROJECT_ROOT" check "$@"
