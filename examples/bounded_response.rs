// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_budget::ResourceBudget;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut budget = ResourceBudget::new("body", 8_usize);
    let mut body = Vec::new();
    for chunk in [b"hello".as_slice(), b" world".as_slice()] {
        if budget.try_consume(chunk.len()).is_err() {
            break;
        }
        body.extend_from_slice(chunk);
    }
    assert_eq!(body, b"hello");
    assert_eq!(budget.used(), 5);
    Ok(())
}
