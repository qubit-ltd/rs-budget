// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Private owned and borrowed decode-session storage.

use crate::ResourceBudget;
use crate::ResourceQuantity;
use crate::json::JsonValueBudget;

/// Backing storage for owned and caller-borrowed decode budgets.
#[derive(Debug)]
pub(crate) enum DecodeStorage<'a, R, Q>
where
    Q: ResourceQuantity,
{
    /// Session-owned budget values.
    Owned {
        /// Optional raw-input budget.
        input: Option<ResourceBudget<R, Q>>,
        /// Optional normalized-input budget.
        normalized_input: Option<ResourceBudget<R, Q>>,
        /// JSON value budget.
        value: JsonValueBudget<R, Q>,
    },
    /// Caller-owned budget references.
    Borrowed {
        /// Optional raw-input budget.
        input: Option<&'a mut ResourceBudget<R, Q>>,
        /// Optional normalized-input budget.
        normalized_input: Option<&'a mut ResourceBudget<R, Q>>,
        /// JSON value budget.
        value: &'a mut JsonValueBudget<R, Q>,
    },
}
