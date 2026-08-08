// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use qubit_budget::LimitExceeded;
use qubit_budget::ResourceBudget;
use qubit_budget::ResourceLimit;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Nodes,
}

#[test]
fn accumulates_and_reports_remaining_capacity() {
    let mut budget = ResourceBudget::new(ResourceLimit::new(10));
    assert_eq!(budget.limit(), ResourceLimit::new(10));
    assert_eq!(budget.used(), 0);
    assert_eq!(budget.remaining(), 10);
    budget.consume(Kind::Nodes, 4).unwrap();
    assert_eq!(budget.used(), 4);
    assert_eq!(budget.remaining(), 6);
    assert!(!budget.is_exhausted());
    budget.consume(Kind::Nodes, 6).unwrap();
    assert!(budget.is_exhausted());
}

#[test]
fn failed_consumption_is_atomic() {
    let mut budget = ResourceLimit::new(10).budget();
    budget.consume(Kind::Nodes, 8).unwrap();
    assert_eq!(
        budget.consume(Kind::Nodes, 3),
        Err(LimitExceeded::new(Kind::Nodes, 10, 11)),
    );
    assert_eq!(budget.used(), 8);
}

#[test]
fn check_additional_does_not_mutate_usage() {
    let budget = ResourceLimit::new(5).budget();
    assert_eq!(
        budget.check_additional(Kind::Nodes, 6),
        Err(LimitExceeded::new(Kind::Nodes, 5, 6)),
    );
    assert_eq!(budget.used(), 0);
}

#[test]
fn overflow_saturates_and_cannot_bypass_the_limit() {
    let mut budget = ResourceLimit::new(usize::MAX - 1).budget();
    budget.consume(Kind::Nodes, usize::MAX - 2).unwrap();
    assert_eq!(
        budget.consume(Kind::Nodes, 10),
        Err(LimitExceeded::new(Kind::Nodes, usize::MAX - 1, usize::MAX,)),
    );
    assert_eq!(budget.used(), usize::MAX - 2);
}
