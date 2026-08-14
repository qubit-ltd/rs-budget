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

/// Verifies dropping an attempt keeps input charges and rolls back value state.
#[test]
fn test_decode_attempt_drop_preserves_input_only() {
    let mut session = JsonDecodeSession::owned(
        JsonDecodeLimits::empty()
            .with_max_input_bytes(4)
            .with_max_nodes(1),
    );
    {
        let mut attempt = session.begin_value();
        attempt
            .try_consume_input_bytes(2)
            .expect("input charge fits");
        attempt
            .try_admit(JsonMeasurement::Null { depth: 1 })
            .expect("value admission fits");
    }

    assert_eq!(session.input_budget().expect("input budget").used(), 2);
    assert_eq!(session.value_budget().used_nodes(), Some(0));
}
