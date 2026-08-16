// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Private owned and borrowed encode-session storage.
// qubit-style: allow source-test-pair

use crate::ResourceBudget;
use crate::ResourceQuantity;
use crate::json::JsonValueBudget;

/// Backing storage for owned and caller-borrowed encode budgets.
#[derive(Debug)]
pub(crate) enum EncodeStorage<'a, R, Q>
where
    Q: ResourceQuantity,
{
    /// Session-owned budget values.
    Owned {
        /// Optional output-byte budget.
        output: Option<ResourceBudget<R, Q>>,
        /// JSON value budget.
        value: JsonValueBudget<R, Q>,
    },
    /// Caller-owned budget references.
    Borrowed {
        /// Optional output-byte budget.
        output: Option<&'a mut ResourceBudget<R, Q>>,
        /// JSON value budget.
        value: &'a mut JsonValueBudget<R, Q>,
    },
}

impl<R, Q> EncodeStorage<'_, R, Q>
where
    Q: ResourceQuantity,
{
    /// Splits storage into the budgets borrowed by one encode attempt.
    #[inline]
    pub(crate) fn split(
        &mut self,
    ) -> (
        Option<&mut ResourceBudget<R, Q>>,
        &mut JsonValueBudget<R, Q>,
    ) {
        match self {
            Self::Owned { output, value } => (output.as_mut(), value),
            Self::Borrowed { output, value } => (output.as_deref_mut(), value),
        }
    }
}
