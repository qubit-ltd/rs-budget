// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Public error-shape coverage for transactional string rendering.

use qubit_budget::BudgetedStringError;

#[test]
fn test_budgeted_string_error_debug_representation_is_stable_enough_to_inspect()
{
    let error = BudgetedStringError::<(), &'static str>::LengthOverflow;
    assert_eq!(format!("{error:?}"), "LengthOverflow");
}
