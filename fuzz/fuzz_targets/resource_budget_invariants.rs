// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
#![no_main]

use libfuzzer_sys::fuzz_target;
use qubit_budget::ResourceBudget;

const MAX_INPUT_LEN: usize = 4 * 1024;

fuzz_target!(|data: &[u8]| {
    let input = &data[..data.len().min(MAX_INPUT_LEN)];
    let maximum = u64::from(input.first().copied().unwrap_or_default());
    let mut budget = ResourceBudget::new((), maximum);
    for chunk in input.get(1..).unwrap_or_default().chunks(2) {
        let requested = u64::from(chunk.first().copied().unwrap_or_default());
        let before = budget.remaining();
        let result = budget.try_consume(requested);
        if result.is_err() {
            assert_eq!(budget.remaining(), before);
        }
        assert!(budget.remaining() <= maximum);
        assert_eq!(budget.used() + budget.remaining(), maximum);
    }
});
