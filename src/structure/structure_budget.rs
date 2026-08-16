// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Enforces structural limits during one processing session.

use super::StructureLimits;
use super::StructureResource;
use crate::resource::BudgetError;
use crate::resource::ResourceBudget;
use crate::resource::ResourceQuantity;
use crate::resource::check_limit;

/// Mutable structural accounting for one processing session.
///
/// `R` and `Q` mirror [`StructureLimits`]. Point limits do not accumulate
/// between calls; node charges consume the session's finite node budget.
#[derive(Debug, PartialEq, Eq)]
pub struct StructureBudget<R = StructureResource, Q = usize>
where
    Q: ResourceQuantity,
{
    /// Immutable point limits for this session.
    limits: StructureLimits<R, Q>,

    /// Optional cumulative node budget for this session.
    nodes: Option<ResourceBudget<R, Q>>,
}

impl<R, Q> StructureBudget<R, Q>
where
    R: Clone,
    Q: ResourceQuantity,
{
    /// Creates a fresh budget session from one structural limit configuration.
    #[inline]
    #[must_use]
    pub(crate) fn new(limits: StructureLimits<R, Q>) -> Self {
        Self {
            nodes: limits
                .nodes_limit()
                .cloned()
                .map(ResourceBudget::from_limit),
            limits,
        }
    }

    /// Checks one nesting depth against its configured maximum.
    #[inline]
    pub fn check_depth(&self, actual: Q) -> Result<(), BudgetError<R, Q>> {
        check_limit(self.limits.depth_limit(), actual)
    }

    /// Charges one processed node to this session's cumulative node budget.
    #[inline]
    pub fn charge_node(&mut self) -> Result<(), BudgetError<R, Q>> {
        self.charge_nodes(Q::ONE)
    }

    /// Charges several processed nodes atomically.
    #[inline]
    pub fn charge_nodes(&mut self, amount: Q) -> Result<(), BudgetError<R, Q>> {
        match &mut self.nodes {
            Some(nodes) => nodes.try_consume(amount).map_err(BudgetError::from),
            None => Ok(()),
        }
    }

    /// Checks one sequence item count against its configured maximum.
    #[inline]
    pub fn check_sequence_items(&self, actual: Q) -> Result<(), BudgetError<R, Q>> {
        check_limit(self.limits.sequence_items_limit(), actual)
    }

    /// Checks one map entry count against its configured maximum.
    #[inline]
    pub fn check_map_entries(&self, actual: Q) -> Result<(), BudgetError<R, Q>> {
        check_limit(self.limits.map_entries_limit(), actual)
    }

    /// Checks one structural key byte length against its configured maximum.
    #[inline]
    pub fn check_key_bytes(&self, actual: Q) -> Result<(), BudgetError<R, Q>> {
        check_limit(self.limits.key_bytes_limit(), actual)
    }

    /// Checks a value depth and charges one node as one atomic traversal step.
    #[inline]
    pub fn enter_node(&mut self, depth: Q) -> Result<(), BudgetError<R, Q>> {
        self.check_depth(depth)?;
        self.charge_node()
    }

    /// Checks a sequence size and charges one node as one atomic traversal
    /// step.
    #[inline]
    pub fn enter_sequence(&mut self, depth: Q, items: Q) -> Result<(), BudgetError<R, Q>> {
        self.check_depth(depth)?;
        self.check_sequence_items(items)?;
        self.charge_node()
    }

    /// Checks a map size and charges one node as one atomic traversal step.
    #[inline]
    pub fn enter_map(&mut self, depth: Q, entries: Q) -> Result<(), BudgetError<R, Q>> {
        self.check_depth(depth)?;
        self.check_map_entries(entries)?;
        self.charge_node()
    }

    /// Returns the immutable limits copied into this session.
    #[must_use]
    #[inline(always)]
    pub const fn limits(&self) -> &StructureLimits<R, Q> {
        &self.limits
    }

    /// Returns the number of nodes consumed by this session.
    #[inline(always)]
    #[must_use]
    pub fn used_nodes(&self) -> Q {
        match &self.nodes {
            Some(nodes) => nodes.used(),
            None => Q::ZERO,
        }
    }
}
