// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Enforces structural limits during one processing session.

use crate::BudgetError;
use crate::ResourceBudget;
use crate::StructureLimits;
use crate::StructureResource;

/// Mutable structural accounting for one processing session.
///
/// Point limits do not accumulate between calls. Node charges consume the
/// session's finite node budget when one was configured.
#[must_use]
#[derive(Debug, PartialEq, Eq)]
pub struct StructureBudget {
    /// Immutable point limits for this session.
    limits: StructureLimits,

    /// Optional cumulative node budget for this session.
    nodes: Option<ResourceBudget<StructureResource, usize>>,
}

impl StructureBudget {
    /// Creates a fresh budget session from one structural limit configuration.
    ///
    /// # Parameters
    ///
    /// * `limits` - Immutable configuration copied into the new session.
    ///
    /// # Returns
    ///
    /// A session whose optional node budget has its full configured capacity.
    #[inline]
    pub(crate) fn new(limits: StructureLimits) -> Self {
        Self {
            nodes: limits.max_nodes.map(ResourceBudget::from_limit),
            limits,
        }
    }

    /// Checks one nesting depth against its configured maximum.
    ///
    /// # Parameters
    ///
    /// * `actual` - Depth of the current value.
    ///
    /// # Returns
    ///
    /// `Ok(())` when depth is unconfigured or fits the inclusive maximum.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetError::LimitExceeded`] with [`StructureResource::Depth`]
    /// when `actual` exceeds the configured maximum. This method does not
    /// mutate the session.
    #[inline]
    pub fn check_depth(
        &self,
        actual: usize,
    ) -> Result<(), BudgetError<StructureResource, usize>> {
        match self.limits.max_depth {
            Some(limit) => limit.check(actual),
            None => Ok(()),
        }
    }

    /// Charges one processed node to this session's cumulative node budget.
    ///
    /// # Returns
    ///
    /// `Ok(())` when nodes are unconfigured or the next node fits.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetError::Insufficient`] with [`StructureResource::Nodes`]
    /// when no node capacity remains. A failed charge leaves the node budget
    /// unchanged.
    #[inline]
    pub fn charge_node(
        &mut self,
    ) -> Result<(), BudgetError<StructureResource, usize>> {
        match &mut self.nodes {
            Some(nodes) => nodes.try_consume(1),
            None => Ok(()),
        }
    }

    /// Checks one sequence item count against its configured maximum.
    ///
    /// # Parameters
    ///
    /// * `actual` - Item count of the current sequence.
    ///
    /// # Returns
    ///
    /// `Ok(())` when sequence items are unconfigured or fit the inclusive
    /// maximum.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetError::LimitExceeded`] with
    /// [`StructureResource::SequenceItems`] when `actual` exceeds the
    /// configured maximum. This method does not mutate the session.
    #[inline]
    pub fn check_sequence_items(
        &self,
        actual: usize,
    ) -> Result<(), BudgetError<StructureResource, usize>> {
        match self.limits.max_sequence_items {
            Some(limit) => limit.check(actual),
            None => Ok(()),
        }
    }

    /// Checks one map entry count against its configured maximum.
    ///
    /// # Parameters
    ///
    /// * `actual` - Entry count of the current map.
    ///
    /// # Returns
    ///
    /// `Ok(())` when map entries are unconfigured or fit the inclusive maximum.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetError::LimitExceeded`] with
    /// [`StructureResource::MapEntries`] when `actual` exceeds the configured
    /// maximum. This method does not mutate the session.
    #[inline]
    pub fn check_map_entries(
        &self,
        actual: usize,
    ) -> Result<(), BudgetError<StructureResource, usize>> {
        match self.limits.max_map_entries {
            Some(limit) => limit.check(actual),
            None => Ok(()),
        }
    }
}
