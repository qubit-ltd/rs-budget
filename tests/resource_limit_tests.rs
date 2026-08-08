// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use qubit_budget::LimitExceeded;
use qubit_budget::ResourceLimit;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Bytes,
}

#[test]
fn accepts_zero_and_exact_boundary() {
    assert_eq!(ResourceLimit::new(0).check(Kind::Bytes, 0), Ok(()));
    assert_eq!(ResourceLimit::new(8).check(Kind::Bytes, 8), Ok(()));
}

#[test]
fn rejects_value_above_boundary_with_the_callers_kind() {
    assert_eq!(
        ResourceLimit::new(8).check(Kind::Bytes, 9),
        Err(LimitExceeded::new(Kind::Bytes, 8, 9)),
    );
}

#[test]
fn exposes_unbounded_limit() {
    let limit = ResourceLimit::unbounded();
    assert!(limit.is_unbounded());
    assert_eq!(limit.maximum(), usize::MAX);
    assert_eq!(limit.check(Kind::Bytes, usize::MAX), Ok(()));
}
