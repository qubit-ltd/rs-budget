// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use qubit_budget::LimitExceeded;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestKind {
    Bytes,
    Items,
}

#[test]
fn exposes_structured_limit_facts() {
    let error = LimitExceeded::new(TestKind::Bytes, 8, 13);
    assert_eq!(error.kind(), &TestKind::Bytes);
    assert_eq!(error.maximum(), 8);
    assert_eq!(error.observed_at_least(), 13);
    assert_eq!(error.into_kind(), TestKind::Bytes);
}

#[test]
fn maps_the_domain_specific_kind() {
    let error =
        LimitExceeded::new(TestKind::Items, 4, 5).map_kind(|kind| match kind {
            TestKind::Bytes => "bytes",
            TestKind::Items => "items",
        });
    assert_eq!(error.kind(), &"items");
    assert_eq!(error.maximum(), 4);
    assert_eq!(error.observed_at_least(), 5);
}
