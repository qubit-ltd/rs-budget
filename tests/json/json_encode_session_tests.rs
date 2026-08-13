// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use qubit_budget::ResourceLimit;
use qubit_budget::json::JsonEncodeLimits;
use qubit_budget::json::JsonEncodeSession;
use qubit_budget::json::JsonResource;

/// Verifies that an owned encode session measures its output budget.
#[test]
fn test_owned_consumes_output_bytes() {
    let limits = JsonEncodeLimits::empty()
        .with_output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, 3_usize));
    let mut session = JsonEncodeSession::owned(limits);

    session.consume_output_bytes_usize(3).expect("output fits");
    assert_eq!(session.output_budget().expect("output budget").used(), 3);
}
