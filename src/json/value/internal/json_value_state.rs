// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Fixed-size committed and working state for JSON value accounting.

use crate::resource::ResourceQuantity;

/// Remaining capacity for the cumulative dimensions of one JSON value budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct JsonValueState<Q>
where
    Q: ResourceQuantity,
{
    /// Remaining capacity for value nodes when that dimension is configured.
    remaining_nodes: Option<Q>,
    /// Remaining capacity for key, string, and number bytes when configured.
    remaining_payload_bytes: Option<Q>,
}

impl<Q> JsonValueState<Q>
where
    Q: ResourceQuantity,
{
    /// Creates the initial zero-used state from configured cumulative maxima.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn new(remaining_nodes: Option<Q>, remaining_payload_bytes: Option<Q>) -> Self {
        Self {
            remaining_nodes,
            remaining_payload_bytes,
        }
    }

    /// Returns remaining value-node capacity when that limit is configured.
    #[inline(always)]
    pub(crate) const fn remaining_nodes(&self) -> Option<Q> {
        self.remaining_nodes
    }

    /// Returns remaining payload-byte capacity when that limit is configured.
    #[inline(always)]
    pub(crate) const fn remaining_payload_bytes(&self) -> Option<Q> {
        self.remaining_payload_bytes
    }

    /// Decreases the checked cumulative capacities for one admitted event.
    pub(crate) fn apply(&mut self, node: bool, payload_bytes: Q) {
        if node && let Some(remaining) = &mut self.remaining_nodes {
            *remaining = *remaining - Q::ONE;
        }
        if let Some(remaining) = &mut self.remaining_payload_bytes {
            *remaining = *remaining - payload_bytes;
        }
    }
}
