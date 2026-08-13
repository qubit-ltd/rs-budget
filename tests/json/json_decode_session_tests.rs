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
