// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_budget::BudgetGroupError;
use qubit_budget::InsufficientBudgetError;
use qubit_budget::ResourceBudget;

#[test]
fn test_group_error_exposes_the_rejecting_budget() {
    let mut first = ResourceBudget::new("bytes", 1_u64);
    let mut second = ResourceBudget::new("bytes", 1_u64);

    let error: BudgetGroupError<&str> =
        ResourceBudget::try_consume_group(&mut [&mut first, &mut second], 2)
            .expect_err("the first budget should reject two bytes");

    assert_eq!(error.index(), 0);
    assert!(matches!(
        error.source_error(),
        InsufficientBudgetError { requested: 2, .. }
    ));
}
