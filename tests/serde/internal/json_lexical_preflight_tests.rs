// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================
//! Tests for lexical JSON admission.

use qubit_budget::JsonDecodeLimits;
use qubit_budget::JsonDecodeSession;
use qubit_budget::decode_slice;

/// Verifies lexical admission accepts one complete JSON value.
#[test]
fn test_json_lexical_preflight_accepts_complete_value() {
    let mut session = JsonDecodeSession::owned(JsonDecodeLimits::empty());
    let value =
        decode_slice::<serde_json::Value, _, _>(br#"{"ok":true}"#, &mut session)
            .expect("complete JSON should pass lexical admission");

    assert_eq!(value["ok"], true);
}
