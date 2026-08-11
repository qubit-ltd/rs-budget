// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::convert::Infallible;
use std::fmt::Write as _;
use std::io::Write as _;

use qubit_budget::BudgetError;
use qubit_budget::BudgetedStringError;
use qubit_budget::ResourceBudget;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestResource {
    OutputBytes,
}

#[test]
fn test_fmt_output_commits_exact_utf8_bytes() {
    let mut budget = ResourceBudget::new(TestResource::OutputBytes, 8_u64);
    let output = budget
        .try_write_string(|writer| {
            let mut output = writer.as_fmt();
            write!(&mut output, "é={}", 7)
        })
        .expect("four UTF-8 bytes should fit");
    assert_eq!(output, "é=7");
    assert_eq!(budget.used(), 4);
    assert_eq!(budget.remaining(), 4);
}

#[test]
fn test_io_output_commits_only_after_success() {
    let mut budget = ResourceBudget::new(TestResource::OutputBytes, 5_u64);
    let output = budget
        .try_write_string(|writer| {
            let mut output = writer.as_io();
            output.write_all(b"hello")
        })
        .expect("exact-limit output should fit");
    assert_eq!(output, "hello");
    assert_eq!(budget.used(), 5);
}

#[test]
fn test_budget_failure_rolls_back_and_reports_first_failing_prefix() {
    let mut budget = ResourceBudget::new(TestResource::OutputBytes, 3_u64);
    let error = budget
        .try_write_string(|writer| {
            let mut output = writer.as_io();
            output.write_all(b"ab")?;
            output.write_all(b"cd")
        })
        .expect_err("the second chunk should exceed the budget");
    assert!(matches!(
        error,
        BudgetedStringError::Budget(BudgetError::Insufficient {
            resource: TestResource::OutputBytes,
            limit: 3,
            remaining: 3,
            requested: 4,
        })
    ));
    assert_eq!(budget.used(), 0);
    assert_eq!(budget.remaining(), 3);
}

#[test]
fn test_render_failure_rolls_back() {
    let mut budget = ResourceBudget::new(TestResource::OutputBytes, 8_u64);
    let error = budget
        .try_write_string(|writer| {
            let mut output = writer.as_io();
            output.write_all(b"ok").expect("prefix should fit");
            Err::<(), _>("renderer failed")
        })
        .expect_err("renderer failure must abort the transaction");
    assert!(matches!(
        error,
        BudgetedStringError::Render("renderer failed")
    ));
    assert_eq!(budget.used(), 0);
}

#[test]
fn test_writer_failure_wins_over_wrapped_render_error() {
    let mut budget = ResourceBudget::new(TestResource::OutputBytes, 1_u64);
    let error = budget
        .try_write_string(|writer| {
            let mut output = writer.as_io();
            output.write_all(b"too long")
        })
        .expect_err("writer should reject the chunk");
    assert!(matches!(error, BudgetedStringError::Budget(_)));
    assert_eq!(budget.used(), 0);
}

#[test]
fn test_invalid_utf8_rolls_back() {
    let mut budget = ResourceBudget::new(TestResource::OutputBytes, 2_u64);
    let error = budget
        .try_write_string(|writer| {
            let mut output = writer.as_io();
            output.write_all(&[0xff])?;
            Ok::<(), std::io::Error>(())
        })
        .expect_err("invalid UTF-8 must not commit");
    assert!(matches!(error, BudgetedStringError::InvalidUtf8(_)));
    assert_eq!(budget.used(), 0);
}

#[test]
fn test_empty_output_is_a_successful_zero_cost_transaction() {
    let mut budget = ResourceBudget::new(TestResource::OutputBytes, 0_u64);
    let output = budget
        .try_write_string(|_| Ok::<(), Infallible>(()))
        .expect("empty output should fit a zero budget");
    assert!(output.is_empty());
    assert_eq!(budget.used(), 0);
}
