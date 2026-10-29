// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Stores committed accounting for independent JSON value transactions.

use super::JsonValueLimits;
use super::JsonValueTransaction;
use super::internal::JsonValueState;
use crate::json::JsonResource;
use crate::resource::ResourceQuantity;

/// Committed JSON value accounting with immutable traversal limits.
///
/// A budget is normally created with [`JsonValueLimits::budget`]. Start a
/// [`JsonValueTransaction`] for each complete value and call `commit` only
/// after every measurement for that value has been admitted.
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource identity retained by limits and errors.
/// * `Q` - Exact unsigned quantity used for measurements and accounting.
///
/// # Examples
///
/// ```
/// use qubit_budget::json::JsonMeasurement;
/// use qubit_budget::json::JsonValueLimits;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut budget = JsonValueLimits::builder()
///     .max_nodes(1_usize)
///     .build()
///     .budget();
/// let mut transaction = budget.transaction();
/// transaction
///     .try_admit(JsonMeasurement::Null { depth: 1 })
///     .expect("the value should fit");
/// transaction.commit()?;
/// assert_eq!(budget.used_nodes(), Some(1));
/// # Ok(()) }
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct JsonValueBudget<R = JsonResource, Q = usize>
where
    Q: ResourceQuantity,
{
    /// Immutable constraints used by every transaction.
    limits: JsonValueLimits<R, Q>,
    /// Accounting published by completed transactions only.
    pub(super) state: JsonValueState<Q>,
}

impl<R, Q> JsonValueBudget<R, Q>
where
    R: Clone,
    Q: ResourceQuantity,
{
    /// Starts an all-or-nothing accounting transaction for one JSON value.
    ///
    /// # Returns
    ///
    /// A transaction that stages admissions against this budget until
    /// committed.
    #[must_use = "the transaction must be committed or dropped intentionally"]
    #[inline]
    pub fn transaction(&mut self) -> JsonValueTransaction<'_, R, Q> {
        JsonValueTransaction::new(self)
    }

    /// Returns the number of nodes published by completed transactions.
    ///
    /// # Returns
    ///
    /// `Some` contains committed node usage when the cumulative node limit is
    /// configured; `None` means node accounting is not configured.
    ///
    /// # Panics
    ///
    /// Panics only if the private accounting state contains a node balance
    /// without the node limit from which it was initialized.
    #[inline]
    pub fn used_nodes(&self) -> Option<Q> {
        self.state
            .remaining_nodes()
            .map(|remaining| self.limits.max_nodes().expect("configured nodes limit") - remaining)
    }

    /// Returns the committed node capacity that remains available.
    ///
    /// # Returns
    ///
    /// `Some` contains remaining node capacity when configured; `None` means
    /// node accounting is not configured.
    #[inline]
    pub const fn remaining_nodes(&self) -> Option<Q> {
        self.state.remaining_nodes()
    }

    /// Returns the payload bytes consumed by completed transactions.
    ///
    /// # Returns
    ///
    /// `Some` contains committed payload usage when the cumulative payload
    /// limit is configured; `None` means payload accounting is not configured.
    ///
    /// # Panics
    ///
    /// Panics only if the private accounting state contains a payload balance
    /// without the payload limit from which it was initialized.
    #[inline]
    pub fn used_payload_bytes(&self) -> Option<Q> {
        self.state
            .remaining_payload_bytes()
            .map(|remaining| self.limits.max_payload_bytes().expect("configured payload limit") - remaining)
    }

    /// Returns the committed payload capacity that remains available.
    ///
    /// # Returns
    ///
    /// `Some` contains remaining payload capacity when configured; `None` means
    /// payload accounting is not configured.
    #[inline]
    pub const fn remaining_payload_bytes(&self) -> Option<Q> {
        self.state.remaining_payload_bytes()
    }
}

impl<R, Q> JsonValueBudget<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates an empty committed ledger using `limits`.
    ///
    /// # Parameters
    ///
    /// * `limits` - Immutable JSON value limits used to initialize committed
    ///   accounting state.
    ///
    /// # Returns
    ///
    /// A budget with zero committed usage and the supplied limits.
    #[inline]
    #[must_use]
    pub fn new(limits: JsonValueLimits<R, Q>) -> Self {
        let state = JsonValueState::new(limits.max_nodes(), limits.max_payload_bytes());
        Self { limits, state }
    }

    /// Borrows the immutable limits shared by all transactions.
    ///
    /// # Returns
    ///
    /// The limits used to initialize this budget and each transaction.
    #[must_use]
    #[inline]
    pub const fn limits(&self) -> &JsonValueLimits<R, Q> {
        &self.limits
    }

    /// Restores the ledger to zero committed usage while preserving its limits.
    pub fn reset(&mut self) {
        self.state = JsonValueState::new(self.limits.max_nodes(), self.limits.max_payload_bytes());
    }
}
