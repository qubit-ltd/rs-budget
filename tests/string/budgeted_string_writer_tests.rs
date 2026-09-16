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

use qubit_budget::BudgetedStringError;
use qubit_budget::InsufficientBudgetError;
use qubit_budget::QuantityMeasurement;
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
            output.write_all(b"hello")?;
            output.flush()
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
        BudgetedStringError::Budget(InsufficientBudgetError {
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
fn test_quantity_failure_preserves_native_measurement_and_budget() {
    let mut budget = ResourceBudget::new(TestResource::OutputBytes, u8::MAX);
    let bytes = [b'x'; u8::MAX as usize + 1];
    let error = budget
        .try_write_string(|writer| {
            let mut output = writer.as_io();
            output.write_all(&bytes)
        })
        .expect_err("an output longer than u8::MAX must fail conversion");

    match error {
        BudgetedStringError::Quantity { resource, source } => {
            assert_eq!(resource, TestResource::OutputBytes);
            assert_eq!(source.measurement(), QuantityMeasurement::Usize(bytes.len()));
            assert_eq!(source.target(), "u8");
        }
        other => panic!("expected a quantity conversion error, got {other:?}"),
    }
    assert_eq!(budget.used(), 0);
    assert_eq!(budget.remaining(), u8::MAX);
}

#[test]
fn test_first_writer_failure_remains_sticky_after_later_writes() {
    let mut budget = ResourceBudget::new(TestResource::OutputBytes, 3_u64);
    let error = budget
        .try_write_string(|writer| {
            let mut output = writer.as_io();
            assert!(
                output.write_all(b"abcd").is_err(),
                "the first write must exceed the budget"
            );
            assert!(
                output.write_all(b"longer").is_err(),
                "later writes must remain rejected"
            );
            Ok::<(), std::io::Error>(())
        })
        .expect_err("the first writer failure must abort the transaction");

    assert!(matches!(
        error,
        BudgetedStringError::Budget(InsufficientBudgetError {
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
    assert!(matches!(error, BudgetedStringError::Render("renderer failed")));
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

#[test]
fn test_writer_supports_usize_budget_quantities() {
    let mut budget = ResourceBudget::new(TestResource::OutputBytes, 4_usize);
    let output = budget
        .try_write_string(|writer| {
            let mut output = writer.as_io();
            output.write_all(b"rust")
        })
        .expect("usize quantity must accept fitting output");
    assert_eq!(output, "rust");
    assert_eq!(budget.used(), 4);
}
