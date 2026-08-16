// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines failures from atomic grouped budget consumption.

use std::fmt::Debug;

use thiserror::Error;

use crate::resource::InsufficientBudgetError;

/// Failure returned by an atomic grouped budget consumption.
///
/// The index identifies the first budget, in caller-provided order, that
/// rejected the request. No member of the group is charged when this error is
/// returned.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("budget group member {index} rejected consumption: {source}")]
pub struct BudgetGroupError<R, Q = u64>
where
    Q: Copy + Debug,
{
    /// Zero-based index of the first rejecting budget.
    index: usize,

    /// Structured failure returned by that budget.
    #[source]
    source: InsufficientBudgetError<R, Q>,
}

impl<R, Q> BudgetGroupError<R, Q>
where
    Q: Copy + Debug,
{
    /// Creates a grouped failure for the first rejecting budget.
    #[must_use]
    pub(crate) const fn new(
        index: usize,
        source: InsufficientBudgetError<R, Q>,
    ) -> Self {
        Self { index, source }
    }

    /// Returns the zero-based index of the first rejecting budget.
    #[inline(always)]
    #[must_use]
    pub const fn index(&self) -> usize {
        self.index
    }

    /// Returns the structured failure from the rejecting budget.
    #[inline(always)]
    #[must_use]
    pub const fn source_error(&self) -> &InsufficientBudgetError<R, Q> {
        &self.source
    }

    /// Consumes this error and returns the rejecting budget's failure.
    #[inline(always)]
    #[must_use]
    pub fn into_source_error(self) -> InsufficientBudgetError<R, Q> {
        self.source
    }
}
