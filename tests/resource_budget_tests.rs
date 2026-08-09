// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use qubit_budget::InvalidRelease;
use qubit_budget::LimitExceeded;
use qubit_budget::ResourceBudget;
use qubit_budget::ResourceLimit;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Nodes,
}

/// Verifies successful consumption and state queries.
#[test]
fn accumulates_and_reports_remaining_capacity() {
    let mut budget = ResourceBudget::new(Kind::Nodes, 10);
    assert_eq!(*budget.resource(), Kind::Nodes);
    assert_eq!(budget.limit(), ResourceLimit::new(10));
    assert_eq!(budget.maximum(), 10);
    assert_eq!(budget.used(), 0);
    assert_eq!(budget.remaining(), 10);
    budget.try_consume(4).unwrap();
    assert_eq!(budget.used(), 4);
    assert_eq!(budget.remaining(), 6);
    assert!(!budget.is_empty());
    assert!(!budget.is_unused());
    budget.try_consume(6).unwrap();
    assert!(budget.is_empty());
}

/// Verifies that failed atomic consumption leaves the budget unchanged.
#[test]
fn failed_consumption_is_atomic() {
    let mut budget = ResourceBudget::new(Kind::Nodes, 10);
    budget.try_consume(8).unwrap();
    assert_eq!(
        budget.try_consume(3),
        Err(LimitExceeded::new(Kind::Nodes, 10, 11)),
    );
    assert_eq!(budget.used(), 8);
}

/// Verifies that additional checks do not mutate usage.
#[test]
fn check_additional_does_not_mutate_usage() {
    let budget = ResourceBudget::new(Kind::Nodes, 5);
    assert_eq!(
        budget.check_additional(6),
        Err(LimitExceeded::new(Kind::Nodes, 5, 6)),
    );
    assert_eq!(budget.used(), 0);
}

/// Verifies that oversized atomic consumption reports saturated observations.
#[test]
fn overflow_saturates_and_cannot_bypass_the_limit() {
    let mut budget = ResourceBudget::new(Kind::Nodes, usize::MAX - 1);
    budget.try_consume(usize::MAX - 2).unwrap();
    assert_eq!(
        budget.try_consume(10),
        Err(LimitExceeded::new(Kind::Nodes, usize::MAX - 1, usize::MAX,)),
    );
    assert_eq!(budget.used(), usize::MAX - 2);
}

/// Verifies that failed exhausting consumption clears the remaining capacity.
#[test]
fn exhausting_consumption_clears_remaining_capacity() {
    let mut budget = ResourceBudget::new(Kind::Nodes, 10);
    budget.try_consume(8).unwrap();
    assert_eq!(
        budget.consume_or_exhaust(3),
        Err(LimitExceeded::new(Kind::Nodes, 10, 11)),
    );
    assert_eq!(budget.remaining(), 0);
}

/// Verifies that partial consumption returns the amount actually consumed.
#[test]
fn partial_consumption_stops_at_remaining_capacity() {
    let mut budget = ResourceBudget::new(Kind::Nodes, 10);
    assert_eq!(budget.consume_available(4), 4);
    assert_eq!(budget.consume_available(10), 6);
    assert_eq!(budget.consume_available(1), 0);
    assert_eq!(budget.used(), 10);
}

/// Verifies that releasing consumed capacity restores availability.
#[test]
fn release_restores_capacity() {
    let mut budget = ResourceBudget::new(Kind::Nodes, 10);
    budget.try_consume(7).unwrap();
    budget.release(4).unwrap();
    assert_eq!(budget.used(), 3);
    assert_eq!(budget.remaining(), 7);
}

/// Verifies that an excessive release is atomic and structured.
#[test]
fn excessive_release_is_atomic() {
    let mut budget = ResourceBudget::new(Kind::Nodes, 10);
    budget.try_consume(3).unwrap();
    assert_eq!(budget.release(4), Err(InvalidRelease::new(3, 4)),);
    assert_eq!(budget.used(), 3);
}

/// Verifies that exhausting a budget returns and discards the remaining
/// capacity.
#[test]
fn exhaust_returns_remaining_capacity() {
    let mut budget = ResourceBudget::new(Kind::Nodes, 10);
    budget.try_consume(3).unwrap();
    assert_eq!(budget.exhaust(), 7);
    assert_eq!(budget.exhaust(), 0);
    assert!(budget.is_empty());
}
