// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::io::Write;

use qubit_budget::ResourceBudget;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut budget = ResourceBudget::new("text", 5_u64);
    let output = budget.try_write_string(|writer| writer.as_io().write_all(b"hello"))?;
    assert_eq!(output, "hello");
    assert_eq!(budget.used(), 5);

    let error = budget.try_write_string(|writer| writer.as_io().write_all(b"!"));
    assert!(error.is_err());
    assert_eq!(budget.used(), 5);
    Ok(())
}
