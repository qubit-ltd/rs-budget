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
///
/// Obtain a `StructureBudget` from [`StructureLimits::budget`] after building
/// the limits. The budget is intended to be kept for one processing session.
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource identity retained by limits and errors.
/// * `Q` - Exact unsigned quantity used for measurements and accounting.
///
/// # Examples
///
/// ```
/// use qubit_budget::StructureLimits;
///
/// let limits = StructureLimits::builder().max_nodes(2).build();
/// let mut budget = limits.budget();
/// budget.charge_node().expect("first node should fit");
/// budget.charge_node().expect("second node should fit");
/// assert_eq!(budget.used_nodes(), Some(2));
/// assert!(budget.charge_node().is_err());
/// ```
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
    ///
    /// # Parameters
    ///
    /// * `limits` - Immutable structural limits used to initialize this
    ///   accounting session.
    ///
    /// # Returns
    ///
    /// Creates a fresh budget session from one structural limit configuration.
    #[inline]
    #[must_use]
    pub(crate) fn new(limits: StructureLimits<R, Q>) -> Self {
        Self {
            nodes: limits.nodes_limit().cloned().map(ResourceBudget::from_limit),
            limits,
        }
    }

    /// Checks one nesting depth against its configured maximum.
    ///
    /// # Parameters
    ///
    /// * `actual` - Observed nesting depth to compare with the configured depth
    ///   limit.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the operation completes successfully.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetError::LimitExceeded`] when a configured depth limit
    /// rejects `actual`.
    #[inline]
    pub fn check_depth(&self, actual: Q) -> Result<(), BudgetError<R, Q>> {
        check_limit(self.limits.depth_limit(), actual)
    }

    /// Charges one processed node to this session's cumulative node budget.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the operation completes successfully.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetError::Insufficient`] when the configured node budget
    /// has fewer than one unit remaining.
    #[inline]
    pub fn charge_node(&mut self) -> Result<(), BudgetError<R, Q>> {
        self.charge_nodes(Q::ONE)
    }

    /// Charges several processed nodes atomically.
    ///
    /// # Parameters
    ///
    /// * `amount` - Number of nodes to charge from the cumulative node budget.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the operation completes successfully.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetError::Insufficient`] when the configured node budget
    /// has fewer than `amount` units remaining.
    #[inline]
    pub fn charge_nodes(&mut self, amount: Q) -> Result<(), BudgetError<R, Q>> {
        match &mut self.nodes {
            Some(nodes) => nodes.try_consume(amount).map_err(BudgetError::from),
            None => Ok(()),
        }
    }

    /// Checks one sequence item count against its configured maximum.
    ///
    /// # Parameters
    ///
    /// * `actual` - Observed number of direct items in the sequence.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the operation completes successfully.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetError::LimitExceeded`] when a configured sequence-item
    /// limit rejects `actual`.
    #[inline]
    pub fn check_sequence_items(&self, actual: Q) -> Result<(), BudgetError<R, Q>> {
        check_limit(self.limits.sequence_items_limit(), actual)
    }

    /// Checks one map entry count against its configured maximum.
    ///
    /// # Parameters
    ///
    /// * `actual` - Observed number of direct entries in the map.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the operation completes successfully.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetError::LimitExceeded`] when a configured map-entry
    /// limit rejects `actual`.
    #[inline]
    pub fn check_map_entries(&self, actual: Q) -> Result<(), BudgetError<R, Q>> {
        check_limit(self.limits.map_entries_limit(), actual)
    }

    /// Checks one structural key byte length against its configured maximum.
    ///
    /// # Parameters
    ///
    /// * `actual` - Observed UTF-8 byte length of the structural key.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the operation completes successfully.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetError::LimitExceeded`] when a configured key-byte limit
    /// rejects `actual`.
    #[inline]
    pub fn check_key_bytes(&self, actual: Q) -> Result<(), BudgetError<R, Q>> {
        check_limit(self.limits.key_bytes_limit(), actual)
    }

    /// Checks a value depth and charges one node as one atomic traversal step.
    ///
    /// # Parameters
    ///
    /// * `depth` - Root-inclusive nesting depth to validate.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the operation completes successfully.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetError::LimitExceeded`] when the depth limit rejects
    /// `depth`, or [`BudgetError::Insufficient`] when the node budget has no
    /// remaining unit.
    #[inline]
    pub fn enter_node(&mut self, depth: Q) -> Result<(), BudgetError<R, Q>> {
        self.check_depth(depth)?;
        self.charge_node()
    }

    /// Checks a sequence size and charges one node as one atomic traversal
    /// step.
    ///
    /// # Parameters
    ///
    /// * `depth` - Root-inclusive nesting depth to validate.
    /// * `items` - Number of direct sequence or array items.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the operation completes successfully.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetError::LimitExceeded`] when the depth or sequence-item
    /// limit rejects its measurement, or [`BudgetError::Insufficient`] when
    /// the node budget has no remaining unit.
    #[inline]
    pub fn enter_sequence(&mut self, depth: Q, items: Q) -> Result<(), BudgetError<R, Q>> {
        self.check_depth(depth)?;
        self.check_sequence_items(items)?;
        self.charge_node()
    }

    /// Checks a map size and charges one node as one atomic traversal step.
    ///
    /// # Parameters
    ///
    /// * `depth` - Root-inclusive nesting depth to validate.
    /// * `entries` - Number of direct map or object entries.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the operation completes successfully.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetError::LimitExceeded`] when the depth or map-entry
    /// limit rejects its measurement, or [`BudgetError::Insufficient`] when
    /// the node budget has no remaining unit.
    #[inline]
    pub fn enter_map(&mut self, depth: Q, entries: Q) -> Result<(), BudgetError<R, Q>> {
        self.check_depth(depth)?;
        self.check_map_entries(entries)?;
        self.charge_node()
    }

    /// Returns the immutable limits copied into this session.
    ///
    /// # Returns
    ///
    /// Returns the immutable limits copied into this session.
    #[must_use]
    #[inline(always)]
    pub const fn limits(&self) -> &StructureLimits<R, Q> {
        &self.limits
    }

    /// Returns whether this session has a finite node limit.
    ///
    /// # Returns
    ///
    /// `true` when the source limits configured a cumulative node maximum.
    #[must_use]
    #[inline(always)]
    pub const fn has_nodes_limit(&self) -> bool {
        self.nodes.is_some()
    }

    /// Returns the node capacity remaining in this session.
    ///
    /// # Returns
    ///
    /// The remaining capacity when a node limit is configured, or `None` for
    /// an unconfigured node dimension.
    #[must_use]
    #[inline(always)]
    pub const fn remaining_nodes(&self) -> Option<Q> {
        match &self.nodes {
            Some(nodes) => Some(nodes.remaining()),
            None => None,
        }
    }

    /// Returns the number of nodes consumed by this session when configured.
    ///
    /// # Returns
    ///
    /// `Some(used)` contains the cumulative node usage when a node limit is
    /// configured. `None` indicates an unconfigured node dimension.
    #[must_use]
    #[inline(always)]
    pub fn used_nodes(&self) -> Option<Q> {
        self.nodes.as_ref().map(ResourceBudget::used)
    }
}
