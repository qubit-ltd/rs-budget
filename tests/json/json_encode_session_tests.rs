// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use std::panic::AssertUnwindSafe;

use qubit_budget::ResourceBudget;
use qubit_budget::json::JsonEncodeLimits;
use qubit_budget::json::JsonEncodeSession;
use qubit_budget::json::JsonMeasurement;
use qubit_budget::json::JsonResource;
use qubit_budget::json::JsonValueLimits;

/// Verifies that dropping an owned attempt retains accepted output but rolls
/// back staged JSON value accounting.
#[test]
fn test_encode_attempt_drop_keeps_output_and_rolls_back_value() {
    let mut session = JsonEncodeSession::owned(
        JsonEncodeLimits::<JsonResource, usize>::new()
            .with_max_output_bytes(8)
            .with_max_nodes(2),
    );
    {
        let mut attempt = session.begin_value();
        attempt.check_output_bytes(3).expect("output fits");
        attempt.try_consume_output_bytes(3).expect("output fits");
        attempt
            .try_admit(JsonMeasurement::Null { depth: 1 })
            .expect("value fits");
    }
    assert_eq!(
        session.output_budget().expect("configured output").used(),
        3
    );
    assert_eq!(session.value_budget().used_nodes(), Some(0));
}

/// Verifies that output checks do not charge output and that an owned session
/// can commit successive value attempts.
#[test]
fn test_encode_attempt_checks_output_and_reuses_session() {
    let mut session = JsonEncodeSession::owned(
        JsonEncodeLimits::<JsonResource, usize>::new()
            .with_max_output_bytes(4)
            .with_max_nodes(2),
    );

    let mut first = session.begin_value();
    first.check_output_bytes(4).expect("output fits");
    assert_eq!(first.output_budget().expect("configured output").used(), 0);
    first.try_consume_output_bytes(3).expect("output fits");
    first
        .try_admit(JsonMeasurement::Null { depth: 1 })
        .expect("value fits");
    assert_eq!(first.used_nodes(), Some(1));
    first.commit();

    let mut second = session.begin_value();
    second
        .try_admit(JsonMeasurement::Null { depth: 1 })
        .expect("value fits");
    second.commit();

    assert_eq!(
        session.output_budget().expect("configured output").used(),
        3
    );
    assert_eq!(session.value_budget().used_nodes(), Some(2));
}

/// Verifies that an attempt exposes its underlying working transaction.
#[test]
fn test_encode_attempt_value_transaction_mut_exposes_working_state() {
    let mut session = JsonEncodeSession::owned(
        JsonEncodeLimits::<JsonResource, usize>::new()
            .with_max_payload_bytes(4),
    );
    let mut attempt = session.begin_value();
    attempt
        .value_transaction_mut()
        .try_admit(JsonMeasurement::String { depth: 1, bytes: 3 })
        .expect("string fits");
    assert_eq!(attempt.used_payload_bytes(), Some(3));
    assert_eq!(attempt.remaining_payload_bytes(), Some(1));
    attempt.commit();
    assert_eq!(session.value_budget().used_payload_bytes(), Some(3));
}

/// Verifies that an attempt can lend output and value accounting together.
#[test]
fn test_encode_attempt_split_mut_allows_output_and_value_accounting() {
    let mut session = JsonEncodeSession::owned(
        JsonEncodeLimits::<JsonResource, usize>::new()
            .with_max_output_bytes(4)
            .with_max_nodes(1),
    );
    let mut attempt = session.begin_value();
    let (output, value) = attempt.split_mut();
    output
        .expect("configured output")
        .try_consume_usize(4)
        .expect("output fits");
    value
        .try_admit(JsonMeasurement::Null { depth: 1 })
        .expect("value fits");
    attempt.commit();

    assert_eq!(
        session.output_budget().expect("configured output").used(),
        4
    );
    assert_eq!(session.value_budget().used_nodes(), Some(1));
}

/// Verifies that a session borrowing only value accounting can commit an
/// attempt while ignoring unconfigured output accounting.
#[test]
fn test_encode_session_borrowing_value_reuses_committed_budget() {
    let mut value = JsonValueLimits::<JsonResource, usize>::new()
        .with_max_nodes(2)
        .budget();
    {
        let mut session = JsonEncodeSession::borrowing_value(&mut value);
        let mut attempt = session.begin_value();
        attempt
            .try_consume_output_bytes(99)
            .expect("unconfigured output is ignored");
        attempt
            .try_admit(JsonMeasurement::Null { depth: 1 })
            .expect("value fits");
        attempt.commit();
    }
    assert_eq!(value.used_nodes(), Some(1));
}

/// Verifies that a session borrowing output retains accepted output when its
/// value attempt is dropped.
#[test]
fn test_encode_session_borrowing_output_keeps_charge_after_attempt_drop() {
    let mut output = ResourceBudget::new(JsonResource::OutputBytes, 3_usize);
    let mut value = JsonValueLimits::<JsonResource, usize>::new()
        .with_max_nodes(2)
        .budget();
    {
        let mut session =
            JsonEncodeSession::borrowing_output(&mut output, &mut value);
        let mut attempt = session.begin_value();
        attempt.try_consume_output_bytes(3).expect("output fits");
        attempt
            .try_admit(JsonMeasurement::Null { depth: 1 })
            .expect("value fits");
    }
    assert_eq!(output.used(), 3);
    assert_eq!(value.used_nodes(), Some(0));
}

/// Verifies that an output error leaves both output and the staged value state
/// unchanged.
#[test]
fn test_encode_attempt_output_error_is_atomic() {
    let mut session = JsonEncodeSession::owned(
        JsonEncodeLimits::<JsonResource, usize>::new()
            .with_max_output_bytes(2)
            .with_max_nodes(1),
    );
    let mut attempt = session.begin_value();
    assert!(attempt.try_consume_output_bytes(3).is_err());
    assert_eq!(
        attempt.output_budget().expect("configured output").used(),
        0
    );
    attempt
        .try_admit(JsonMeasurement::Null { depth: 1 })
        .expect("value fits");
    drop(attempt);
    assert_eq!(session.value_budget().used_nodes(), Some(0));
}

/// Verifies that unwinding drops an encode attempt without publishing its
/// value state while preserving already accepted output.
#[test]
fn test_encode_attempt_panic_keeps_output_and_rolls_back_value() {
    let mut session = JsonEncodeSession::owned(
        JsonEncodeLimits::<JsonResource, usize>::new()
            .with_max_output_bytes(3)
            .with_max_nodes(1),
    );
    let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
        let mut attempt = session.begin_value();
        attempt.try_consume_output_bytes(3).expect("output fits");
        attempt
            .try_admit(JsonMeasurement::Null { depth: 1 })
            .expect("value fits");
        panic!("abort encode after accounting");
    }));
    assert!(result.is_err());
    assert_eq!(
        session.output_budget().expect("configured output").used(),
        3
    );
    assert_eq!(session.value_budget().used_nodes(), Some(0));
}
