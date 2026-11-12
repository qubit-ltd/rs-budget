// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Verifies decode-attempt I/O and value accounting boundaries.

use qubit_budget::json::JsonDecodeLimits;
use qubit_budget::json::JsonDecodeSession;
use qubit_budget::json::JsonMeasurement;
use qubit_budget::json::JsonResource;

/// Verifies dropping an attempt keeps input charges and rolls back value state.
#[test]
fn test_decode_attempt_drop_preserves_input_only() {
    let mut session = JsonDecodeSession::from_limits(
        JsonDecodeLimits::<JsonResource, usize>::builder()
            .max_input_bytes(4)
            .max_nodes(1)
            .build(),
    );
    {
        let mut attempt = session.begin_value();
        attempt.try_consume_input_bytes(2).expect("input charge fits");
        attempt
            .try_admit(JsonMeasurement::Null { depth: 1 })
            .expect("value admission fits");
    }

    assert_eq!(session.input_budget().expect("input budget").used(), 2);
    assert_eq!(session.value_budget().used_nodes(), Some(0));
}

#[test]
fn test_decode_attempt_can_consume_input_bytes() {
    let mut session = JsonDecodeSession::from_limits(
        JsonDecodeLimits::<JsonResource, usize>::builder()
            .max_input_bytes(4)
            .max_normalized_input_bytes(4)
            .build(),
    );
    let mut attempt = session.begin_value();
    attempt.try_consume_input_bytes(3).expect("input charge fits");
    attempt
        .try_consume_normalized_input_bytes(2)
        .expect("normalized input charge fits");
    assert_eq!(attempt.input_budget().expect("input budget").used(), 3);
    assert_eq!(
        attempt
            .normalized_input_budget()
            .expect("normalized input budget")
            .used(),
        2
    );
}

/// Verifies attempts forward cumulative value usage and drop only new staging.
#[test]
fn test_decode_attempt_usage_includes_committed_value_state() {
    let mut session = JsonDecodeSession::from_limits(
        JsonDecodeLimits::<JsonResource, usize>::builder()
            .max_nodes(8)
            .max_payload_bytes(16)
            .build(),
    );
    {
        let mut attempt = session.begin_value();
        attempt
            .try_admit(JsonMeasurement::String { depth: 1, bytes: 2 })
            .expect("first string admission fits");
        attempt.commit().expect("first attempt commits");
    }
    {
        let mut attempt = session.begin_value();
        assert_eq!((attempt.used_nodes(), attempt.used_payload_bytes()), (Some(1), Some(2)));
        attempt
            .try_admit(JsonMeasurement::String { depth: 1, bytes: 3 })
            .expect("second string admission fits");
        assert_eq!((attempt.used_nodes(), attempt.used_payload_bytes()), (Some(2), Some(5)));
    }
    assert_eq!(session.value_budget().used_nodes(), Some(1));
    assert_eq!(session.value_budget().used_payload_bytes(), Some(2));
}
