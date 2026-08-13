// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use qubit_budget::ResourceBudget;
use qubit_budget::ResourceLimit;
use qubit_budget::json::JsonEncodeLimits;
use qubit_budget::json::JsonEncodeSession;
use qubit_budget::json::JsonResource;
use qubit_budget::json::JsonValueBudget;
use qubit_budget::json::JsonValueLimits;

/// Verifies that an owned encode session measures its output budget.
#[test]
fn test_owned_consumes_output_bytes() {
    let limits = JsonEncodeLimits::empty().with_output_bytes_limit(
        ResourceLimit::new(JsonResource::OutputBytes, 3_usize),
    );
    let mut session = JsonEncodeSession::owned(limits);

    session.consume_output_bytes_usize(3).expect("output fits");
    assert_eq!(session.output_budget().expect("output budget").used(), 3);
}

/// Verifies that each borrowed encode constructor charges only its supplied
/// budgets.
#[test]
fn test_borrowing_constructors_charge_only_supplied_dimensions() {
    let mut output = ResourceBudget::new(JsonResource::OutputBytes, 3_usize);
    let mut value = JsonValueBudget::new(JsonValueLimits::empty());

    JsonEncodeSession::borrowing_value(&mut value)
        .consume_output_bytes_usize(99)
        .expect("output budget is absent");
    JsonEncodeSession::borrowing_output(&mut output, &mut value)
        .consume_output_bytes_usize(3)
        .expect("output fits");

    assert_eq!(output.used(), 3);
}
