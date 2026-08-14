// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Stores committed accounting for independent JSON value transactions.

use super::JsonResource;
use super::JsonValueLimits;
use super::JsonValueTransaction;
use super::internal::JsonValueState;
use crate::ResourceQuantity;

/// Committed JSON value accounting with immutable traversal limits.
#[must_use]
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
    /// Creates an empty committed ledger for `limits`.
    pub fn new(limits: JsonValueLimits<R, Q>) -> Self {
        let state =
            JsonValueState::new(limits.max_nodes(), limits.max_payload_bytes());
        Self { limits, state }
    }

    /// Restores the ledger to its original zero-used committed state.
    pub fn reset(&mut self) {
        self.state = JsonValueState::new(
            self.limits.max_nodes(),
            self.limits.max_payload_bytes(),
        );
    }

    /// Starts an all-or-nothing accounting transaction for one JSON value.
    pub fn transaction(&mut self) -> JsonValueTransaction<'_, R, Q> {
        JsonValueTransaction::new(self)
    }

    /// Returns the immutable limits shared by all transactions.
    #[must_use = "the limits determine which transactions can be committed"]
    pub const fn limits(&self) -> &JsonValueLimits<R, Q> {
        &self.limits
    }

    /// Returns committed node usage when the cumulative node limit is set.
    #[must_use]
    pub fn used_nodes(&self) -> Option<Q> {
        self.state.remaining_nodes().map(|remaining| {
            self.limits.max_nodes().expect("configured nodes limit") - remaining
        })
    }

    /// Returns committed remaining node capacity when that limit is set.
    #[must_use]
    pub const fn remaining_nodes(&self) -> Option<Q> {
        self.state.remaining_nodes()
    }

    /// Returns committed payload usage when the cumulative payload limit is
    /// set.
    #[must_use]
    pub fn used_payload_bytes(&self) -> Option<Q> {
        self.state.remaining_payload_bytes().map(|remaining| {
            self.limits
                .max_payload_bytes()
                .expect("configured payload limit")
                - remaining
        })
    }

    /// Returns committed remaining payload capacity when that limit is set.
    #[must_use]
    pub const fn remaining_payload_bytes(&self) -> Option<Q> {
        self.state.remaining_payload_bytes()
    }
}
