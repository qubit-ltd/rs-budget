// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Verifies JSON container kind identity semantics.

use qubit_budget::json::JsonContainerKind;

/// Verifies array and object container dimensions remain distinct.
#[test]
fn test_json_container_kind_variants_are_distinct() {
    assert_ne!(JsonContainerKind::Sequence, JsonContainerKind::Map);
    assert_eq!(JsonContainerKind::Sequence, JsonContainerKind::Sequence);
}
