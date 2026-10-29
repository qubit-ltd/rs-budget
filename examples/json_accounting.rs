// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Demonstrates charging raw and normalized JSON input, admitting measured
//! values, and committing the resulting accounting transaction.

use qubit_budget::json::JsonDecodeLimits;
use qubit_budget::json::JsonDecodeSession;
use qubit_budget::json::JsonMeasurement;
use qubit_budget::json::JsonResource;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut session = JsonDecodeSession::from_limits(
        JsonDecodeLimits::<JsonResource, usize>::builder()
            .max_input_bytes(100)
            .max_normalized_input_bytes(100)
            .max_nodes(4)
            .build(),
    );
    let raw = br#"{"name":"Ada"}"#;
    assert_eq!(raw.len(), 14);
    let mut attempt = session.begin_value();
    attempt.try_consume_input_bytes(raw.len())?;
    attempt.try_consume_normalized_input_bytes(raw.len())?;
    attempt.try_admit(JsonMeasurement::Object { depth: 1, entries: 1 })?;
    attempt.try_admit(JsonMeasurement::Key { bytes: 4 })?;
    attempt.try_admit(JsonMeasurement::String { depth: 2, bytes: 3 })?;
    attempt.commit()?;
    assert_eq!(session.input_budget().unwrap().used(), 14);
    assert_eq!(session.value_budget().used_nodes(), Some(2));
    println!("accepted input=14 nodes=2");
    Ok(())
}
