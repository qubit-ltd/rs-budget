// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Verifies encode-attempt output and value accounting boundaries.

use qubit_budget::json::JsonEncodeLimits;
use qubit_budget::json::JsonEncodeSession;
use qubit_budget::json::JsonMeasurement;
use qubit_budget::json::JsonResource;

/// Verifies dropping an attempt keeps accepted output and rolls back values.
#[test]
fn test_encode_attempt_drop_preserves_output_only() {
    let mut session = JsonEncodeSession::from_limits(
        JsonEncodeLimits::<JsonResource, usize>::builder()
            .max_output_bytes(4)
            .max_nodes(1)
            .build(),
    );
    {
        let mut attempt = session.begin_value();
        attempt.try_consume_output_bytes(2).expect("output charge fits");
        attempt
            .try_admit(JsonMeasurement::Null { depth: 1 })
            .expect("value admission fits");
    }

    assert_eq!(session.output_budget().expect("output budget").used(), 2);
    assert_eq!(session.value_budget().used_nodes(), Some(0));
}

#[test]
fn test_encode_attempt_can_consume_output_bytes() {
    let mut session = JsonEncodeSession::from_limits(
        JsonEncodeLimits::<JsonResource, usize>::builder()
            .max_output_bytes(4)
            .build(),
    );
    let mut attempt = session.begin_value();
    attempt.try_consume_output_bytes(3).expect("output charge fits");
    assert_eq!(attempt.output_budget().expect("output budget").used(), 3);
}

/// Verifies attempts forward cumulative value usage and drop only new staging.
#[test]
fn test_encode_attempt_usage_includes_committed_value_state() {
    let mut session = JsonEncodeSession::from_limits(
        JsonEncodeLimits::<JsonResource, usize>::builder()
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
