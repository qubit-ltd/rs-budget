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

/// Verifies dropping an attempt keeps accepted output and rolls back values.
#[test]
fn test_encode_attempt_drop_preserves_output_only() {
    let mut session = JsonEncodeSession::owned(
        JsonEncodeLimits::empty()
            .with_max_output_bytes(4)
            .with_max_nodes(1),
    );
    {
        let mut attempt = session.begin_value();
        attempt
            .try_consume_output_bytes(2)
            .expect("output charge fits");
        attempt
            .try_admit(JsonMeasurement::Null { depth: 1 })
            .expect("value admission fits");
    }

    assert_eq!(session.output_budget().expect("output budget").used(), 2);
    assert_eq!(session.value_budget().used_nodes(), Some(0));
}
