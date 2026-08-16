// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use std::panic::AssertUnwindSafe;

use qubit_budget::ResourceBudget;
use qubit_budget::json::JsonDecodeLimits;
use qubit_budget::json::JsonDecodeSession;
use qubit_budget::json::JsonMeasurement;
use qubit_budget::json::JsonResource;
use qubit_budget::json::JsonValueLimits;

/// Verifies that dropping an owned attempt retains I/O charges but rolls back
/// staged JSON value accounting.
#[test]
fn test_decode_attempt_drop_keeps_input_and_rolls_back_value() {
    let mut session = JsonDecodeSession::owned(
        JsonDecodeLimits::<JsonResource, usize>::new()
            .with_max_input_bytes(8)
            .with_max_normalized_input_bytes(7)
            .with_max_nodes(2),
    );
    {
        let mut attempt = session.begin_value();
        attempt.try_consume_input_bytes(3).expect("input fits");
        attempt
            .try_consume_normalized_input_bytes(2)
            .expect("normalized input fits");
        attempt
            .try_admit(JsonMeasurement::Null { depth: 1 })
            .expect("value fits");
    }
    assert_eq!(session.input_budget().expect("configured input").used(), 3);
    assert_eq!(
        session
            .normalized_input_budget()
            .expect("configured normalized input")
            .used(),
        2
    );
    assert_eq!(session.value_budget().used_nodes(), Some(0));
}

/// Verifies that an owned decode session can commit successive value attempts.
#[test]
fn test_decode_session_begin_value_commits_and_reuses_value_budget() {
    let mut session =
        JsonDecodeSession::owned(JsonDecodeLimits::<JsonResource, usize>::new().with_max_nodes(2));

    let mut first = session.begin_value();
    first
        .try_admit(JsonMeasurement::Null { depth: 1 })
        .expect("first value fits");
    assert_eq!(first.used_nodes(), Some(1));
    assert_eq!(first.remaining_nodes(), Some(1));
    first.commit();

    let mut second = session.begin_value();
    second
        .try_admit(JsonMeasurement::Null { depth: 1 })
        .expect("second value fits");
    second.commit();

    let mut rejected = session.begin_value();
    assert!(
        rejected
            .try_admit(JsonMeasurement::Null { depth: 1 })
            .is_err()
    );
    assert_eq!(session.value_budget().used_nodes(), Some(2));
}

/// Verifies that an attempt exposes its underlying working transaction.
#[test]
fn test_decode_attempt_value_transaction_mut_exposes_working_state() {
    let mut session = JsonDecodeSession::owned(
        JsonDecodeLimits::<JsonResource, usize>::new().with_max_payload_bytes(4),
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

/// Verifies that a session borrowing only a value budget can reuse it after
/// committed attempts and ignores unconfigured input accounting.
#[test]
fn test_decode_session_borrowing_value_reuses_committed_budget() {
    let mut value = JsonValueLimits::<JsonResource, usize>::new()
        .with_max_nodes(2)
        .budget();
    {
        let mut session = JsonDecodeSession::borrowing_value(&mut value);
        let mut attempt = session.begin_value();
        attempt
            .try_consume_input_bytes(99)
            .expect("unconfigured input is ignored");
        attempt
            .try_admit(JsonMeasurement::Null { depth: 1 })
            .expect("value fits");
        attempt.commit();
    }
    assert_eq!(value.used_nodes(), Some(1));
}

/// Verifies that a session borrowing input keeps its immediate input charge
/// when its value attempt is dropped.
#[test]
fn test_decode_session_borrowing_input_keeps_charge_after_attempt_drop() {
    let mut input = ResourceBudget::new(JsonResource::InputBytes, 3_usize);
    let mut value = JsonValueLimits::<JsonResource, usize>::new()
        .with_max_nodes(2)
        .budget();
    {
        let mut session = JsonDecodeSession::borrowing_input(&mut input, &mut value);
        let mut attempt = session.begin_value();
        attempt.try_consume_input_bytes(3).expect("input fits");
        attempt
            .try_admit(JsonMeasurement::Null { depth: 1 })
            .expect("value fits");
    }
    assert_eq!(input.used(), 3);
    assert_eq!(value.used_nodes(), Some(0));
}

/// Verifies that a session borrowing every decode budget commits all requested
/// accounting dimensions.
#[test]
fn test_decode_session_borrowing_all_commits_io_and_value() {
    let mut input = ResourceBudget::new(JsonResource::InputBytes, 3_usize);
    let mut normalized = ResourceBudget::new(JsonResource::NormalizedInputBytes, 4_usize);
    let mut value = JsonValueLimits::<JsonResource, usize>::new()
        .with_max_nodes(2)
        .budget();
    {
        let mut session = JsonDecodeSession::borrowing_all(&mut input, &mut normalized, &mut value);
        let mut attempt = session.begin_value();
        attempt.try_consume_input_bytes(3).expect("input fits");
        attempt
            .try_consume_normalized_input_bytes(4)
            .expect("normalized input fits");
        attempt
            .try_admit(JsonMeasurement::Null { depth: 1 })
            .expect("value fits");
        attempt.commit();
    }
    assert_eq!(input.used(), 3);
    assert_eq!(normalized.used(), 4);
    assert_eq!(value.used_nodes(), Some(1));
}

/// Verifies that unwinding drops a decode attempt without publishing its value
/// state while preserving already consumed input.
#[test]
fn test_decode_attempt_panic_keeps_input_and_rolls_back_value() {
    let mut session = JsonDecodeSession::owned(
        JsonDecodeLimits::<JsonResource, usize>::new()
            .with_max_input_bytes(3)
            .with_max_nodes(1),
    );
    let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
        let mut attempt = session.begin_value();
        attempt.try_consume_input_bytes(3).expect("input fits");
        attempt
            .try_admit(JsonMeasurement::Null { depth: 1 })
            .expect("value fits");
        panic!("abort decode after accounting");
    }));
    assert!(result.is_err());
    assert_eq!(session.input_budget().expect("configured input").used(), 3);
    assert_eq!(session.value_budget().used_nodes(), Some(0));
}
