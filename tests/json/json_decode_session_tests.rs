// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use qubit_budget::ResourceBudget;
use qubit_budget::ResourceLimit;
use qubit_budget::json::JsonDecodeLimits;
use qubit_budget::json::JsonDecodeSession;
use qubit_budget::json::JsonResource;
use qubit_budget::json::JsonValueBudget;
use qubit_budget::json::JsonValueLimits;

/// Verifies that decode sessions measure raw and normalized input
/// independently.
#[test]
fn test_owned_consumes_raw_and_normalized_input_independently() {
    let limits = JsonDecodeLimits::empty()
        .with_input_bytes_limit(ResourceLimit::new(
            JsonResource::InputBytes,
            3_usize,
        ))
        .with_normalized_input_bytes_limit(ResourceLimit::new(
            JsonResource::NormalizedInputBytes,
            4_usize,
        ));
    let mut session = JsonDecodeSession::owned(limits);

    session
        .consume_input_bytes_usize(3)
        .expect("raw input fits");
    session
        .consume_normalized_input_bytes_usize(4)
        .expect("normalized input fits");
    assert_eq!(session.input_budget().expect("raw budget").used(), 3);
    assert_eq!(
        session
            .normalized_input_budget()
            .expect("normalized budget")
            .used(),
        4,
    );
}

/// Verifies that each borrowed decode constructor charges only its supplied
/// budgets.
#[test]
fn test_borrowing_constructors_charge_only_supplied_dimensions() {
    let mut input = ResourceBudget::new(JsonResource::InputBytes, 2_usize);
    let mut normalized =
        ResourceBudget::new(JsonResource::NormalizedInputBytes, 3_usize);
    let mut value = JsonValueBudget::new(JsonValueLimits::empty());

    JsonDecodeSession::borrowing_value(&mut value)
        .consume_input_bytes_usize(99)
        .expect("input budget is absent");
    JsonDecodeSession::borrowing_input(&mut input, &mut value)
        .consume_input_bytes_usize(2)
        .expect("input fits");
    JsonDecodeSession::borrowing_all(&mut input, &mut normalized, &mut value)
        .consume_normalized_input_bytes_usize(3)
        .expect("normalized input fits");

    assert_eq!(input.used(), 2);
    assert_eq!(normalized.used(), 3);
}

/// Verifies that a decode session exposes and reuses its mutable value budget.
#[test]
fn test_owned_value_budget_remaining_and_reuse() {
    let limits = JsonDecodeLimits::empty().with_max_nodes(2);
    let mut session = JsonDecodeSession::owned(limits);

    session
        .value_budget_mut()
        .enter_node_usize(1)
        .expect("first node fits");
    assert_eq!(session.value_budget().structure_budget().used_nodes(), 1);
    session
        .value_budget_mut()
        .enter_node_usize(1)
        .expect("second node fits");
    assert!(session.value_budget_mut().enter_node_usize(1).is_err());
}

/// Verifies that borrowed sessions write all value charges back to the caller.
#[test]
fn test_borrowed_value_budget_is_reused_after_session_drop() {
    let mut value = JsonValueLimits::empty().with_max_nodes(2).budget();
    {
        let mut session = JsonDecodeSession::borrowing_value(&mut value);
        session
            .value_budget_mut()
            .enter_node_usize(1)
            .expect("borrowed node fits");
    }
    assert_eq!(value.structure_budget().used_nodes(), 1);
    value.enter_node_usize(1).expect("caller can reuse budget");
}

/// Verifies generic and native input accounting share the same budget.
#[test]
fn test_generic_input_consumption_is_atomic() {
    let limits = JsonDecodeLimits::empty().with_input_bytes_limit(
        ResourceLimit::new(JsonResource::InputBytes, 4_usize),
    );
    let mut session = JsonDecodeSession::owned(limits);
    session.consume_input_bytes(2).expect("raw input fits");
    session
        .consume_input_bytes_usize(2)
        .expect("native raw input fits");
    assert_eq!(session.input_budget().expect("input").used(), 4);
    assert!(session.consume_input_bytes(1).is_err());
}

#[test]
fn test_generic_normalized_input_consumption_uses_configured_budget() {
    let limits = JsonDecodeLimits::empty().with_normalized_input_bytes_limit(
        ResourceLimit::new(JsonResource::NormalizedInputBytes, 2_usize),
    );
    let mut session = JsonDecodeSession::owned(limits);
    session
        .consume_normalized_input_bytes(1)
        .expect("normalized input fits");
    assert_eq!(session.max_normalized_input_bytes(), Some(2));
    assert_eq!(
        session
            .normalized_input_budget()
            .expect("normalized input")
            .used(),
        1
    );
}
