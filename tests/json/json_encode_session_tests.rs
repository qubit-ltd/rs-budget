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

/// Verifies that encode sessions expose remaining output capacity and value
/// use.
#[test]
fn test_owned_output_remaining_and_value_budget_reuse() {
    let limits = JsonEncodeLimits::empty()
        .with_max_output_bytes(4)
        .with_max_nodes(2);
    let mut session = JsonEncodeSession::owned(limits);

    session.consume_output_bytes_usize(3).expect("output fits");
    session
        .value_budget_mut()
        .enter_node_usize(1)
        .expect("value node fits");

    assert_eq!(session.max_output_bytes(), Some(4));
    assert_eq!(
        session
            .output_budget()
            .expect("output configured")
            .remaining(),
        1
    );
    assert_eq!(session.value_budget().structure_budget().used_nodes(), 1);
}

/// Verifies that borrowed output and value budgets remain usable after a
/// session.
#[test]
fn test_borrowed_output_and_value_budgets_are_reused_after_session_drop() {
    let mut output = ResourceBudget::new(JsonResource::OutputBytes, 4_usize);
    let mut value = JsonValueLimits::empty().with_max_nodes(2).budget();
    {
        let mut session =
            JsonEncodeSession::borrowing_output(&mut output, &mut value);
        session
            .consume_output_bytes_usize(2)
            .expect("borrowed output fits");
        session
            .value_budget_mut()
            .enter_node_usize(1)
            .expect("borrowed value fits");
    }
    assert_eq!(output.remaining(), 2);
    assert_eq!(value.structure_budget().used_nodes(), 1);
    value
        .enter_node_usize(1)
        .expect("caller can reuse value budget");
}

/// Verifies generic output accounting and the configured-output accessors.
#[test]
fn test_generic_output_consumption_is_atomic() {
    let limits = JsonEncodeLimits::empty().with_output_bytes_limit(
        ResourceLimit::new(JsonResource::OutputBytes, 4_usize),
    );
    let mut session = JsonEncodeSession::owned(limits);
    session.consume_output_bytes(2).expect("output fits");
    session
        .consume_output_bytes_usize(2)
        .expect("native output fits");
    assert_eq!(session.output_budget().expect("output").used(), 4);
    assert!(session.consume_output_bytes(1).is_err());
}
