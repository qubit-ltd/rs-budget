// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Private owned and borrowed decode-session storage.
// qubit-style: allow source-test-pair

use crate::json::JsonValueBudget;
use crate::resource::ResourceBudget;
use crate::resource::ResourceQuantity;

/// Mutable budgets borrowed by one decode attempt.
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource identity retained by limits and errors.
/// * `Q` - Exact unsigned quantity used for measurements and accounting.
type DecodeStorageSplit<'a, R, Q> = (
    Option<&'a mut ResourceBudget<R, Q>>,
    Option<&'a mut ResourceBudget<R, Q>>,
    &'a mut JsonValueBudget<R, Q>,
);

/// Backing storage for owned and caller-borrowed decode budgets.
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource identity retained by limits and errors.
/// * `Q` - Exact unsigned quantity used for measurements and accounting.
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

impl<R, Q> DecodeStorage<'_, R, Q>
where
    Q: ResourceQuantity,
{
    /// Splits storage into the budgets borrowed by one decode attempt.
    ///
    /// # Returns
    ///
    /// Splits storage into the budgets borrowed by one decode attempt.
    ///
    /// A `None` I/O budget indicates that the corresponding byte dimension is
    /// unconfigured.
    #[inline]
    pub(crate) fn split(&mut self) -> DecodeStorageSplit<'_, R, Q> {
        match self {
            Self::Owned {
                input,
                normalized_input,
                value,
            } => (input.as_mut(), normalized_input.as_mut(), value),
            Self::Borrowed {
                input,
                normalized_input,
                value,
            } => (input.as_deref_mut(), normalized_input.as_deref_mut(), value),
        }
    }
}
