// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Builds optional structural input limits.

use super::StructureLimits;
use super::StructureResource;
use crate::resource::ResourceLimit;
use crate::resource::ResourceQuantity;

/// Builder for [`StructureLimits`].
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource identity retained by limits and errors.
/// * `Q` - Exact unsigned quantity used for measurements and accounting.
///
/// # Examples
///
/// ```
/// use qubit_budget::StructureLimitsBuilder;
///
/// let limits = StructureLimitsBuilder::new().max_depth(4).max_nodes(16).build();
/// assert_eq!(limits.max_depth(), Some(4));
/// assert_eq!(limits.max_nodes(), Some(16));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StructureLimitsBuilder<R = StructureResource, Q = usize>
where
    Q: ResourceQuantity,
{
    /// Limit configuration accumulated by chained builder calls.
    limits: StructureLimits<R, Q>,
}

impl<R, Q> Default for StructureLimitsBuilder<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates an empty builder through the standard [`Default`] interface.
    ///
    /// # Returns
    ///
    /// Creates an empty builder through the standard [`Default`] interface.
    fn default() -> Self {
        Self::new()
    }
}

impl<R, Q> From<StructureLimitsBuilder<R, Q>> for StructureLimits<R, Q>
where
    Q: ResourceQuantity,
{
    /// Finishes the builder and returns its structural limit configuration.
    ///
    /// # Parameters
    ///
    /// * `builder` - Builder whose accumulated configuration is consumed.
    ///
    /// # Returns
    ///
    /// Finishes the builder and returns its structural limit configuration.
    fn from(builder: StructureLimitsBuilder<R, Q>) -> Self {
        builder.build()
    }
}

impl<R, Q> StructureLimitsBuilder<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates an empty structural-limits builder.
    ///
    /// # Returns
    ///
    /// Creates an empty structural-limits builder.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            limits: StructureLimits::new(),
        }
    }

    /// Creates a builder retaining an existing limit configuration.
    ///
    /// # Parameters
    ///
    /// * `limits` - Existing structural limits whose configuration is copied
    ///   into this builder.
    ///
    /// # Returns
    ///
    /// Creates a builder retaining an existing limit configuration.
    #[inline]
    #[must_use]
    pub(crate) const fn from_limits(limits: StructureLimits<R, Q>) -> Self {
        Self { limits }
    }

    /// Sets the depth limit.
    ///
    /// # Parameters
    ///
    /// * `limit` - Resource-bound nesting-depth limit to install.
    ///
    /// # Returns
    ///
    /// The builder with the described setting applied.
    #[inline]
    #[must_use]
    pub fn depth_limit(mut self, limit: ResourceLimit<R, Q>) -> Self {
        self.limits.set_depth_limit(limit);
        self
    }

    /// Sets the node limit.
    ///
    /// # Parameters
    ///
    /// * `limit` - Resource-bound cumulative node limit to install.
    ///
    /// # Returns
    ///
    /// The builder with the described setting applied.
    #[inline]
    #[must_use]
    pub fn nodes_limit(mut self, limit: ResourceLimit<R, Q>) -> Self {
        self.limits.set_nodes_limit(limit);
        self
    }

    /// Sets the sequence-item limit.
    ///
    /// # Parameters
    ///
    /// * `limit` - Resource-bound sequence-item limit to install.
    ///
    /// # Returns
    ///
    /// The builder with the described setting applied.
    #[inline]
    #[must_use]
    pub fn sequence_items_limit(mut self, limit: ResourceLimit<R, Q>) -> Self {
        self.limits.set_sequence_items_limit(limit);
        self
    }

    /// Sets the map-entry limit.
    ///
    /// # Parameters
    ///
    /// * `limit` - Resource-bound map-entry limit to install.
    ///
    /// # Returns
    ///
    /// The builder with the described setting applied.
    #[inline]
    #[must_use]
    pub fn map_entries_limit(mut self, limit: ResourceLimit<R, Q>) -> Self {
        self.limits.set_map_entries_limit(limit);
        self
    }

    /// Sets the structural-key limit.
    ///
    /// # Parameters
    ///
    /// * `limit` - Resource-bound structural-key limit to install.
    ///
    /// # Returns
    ///
    /// The builder with the described setting applied.
    #[inline]
    #[must_use]
    pub fn key_bytes_limit(mut self, limit: ResourceLimit<R, Q>) -> Self {
        self.limits.set_key_bytes_limit(limit);
        self
    }

    /// Builds the configured structural limits.
    ///
    /// # Returns
    ///
    /// Builds the configured structural limits.
    #[inline]
    #[must_use]
    pub fn build(self) -> StructureLimits<R, Q> {
        self.limits
    }
}

impl StructureLimitsBuilder<StructureResource, usize> {
    /// Sets the maximum nesting depth.
    ///
    /// # Parameters
    ///
    /// * `maximum` - Inclusive maximum to configure.
    ///
    /// # Returns
    ///
    /// The builder with the described setting applied.
    #[inline]
    #[must_use]
    pub const fn max_depth(mut self, maximum: usize) -> Self {
        self.limits.set_max_depth(maximum);
        self
    }

    /// Sets the maximum number of processed nodes.
    ///
    /// # Parameters
    ///
    /// * `maximum` - Inclusive maximum to configure.
    ///
    /// # Returns
    ///
    /// The builder with the described setting applied.
    #[inline]
    #[must_use]
    pub const fn max_nodes(mut self, maximum: usize) -> Self {
        self.limits.set_max_nodes(maximum);
        self
    }

    /// Sets the maximum number of items in one sequence.
    ///
    /// # Parameters
    ///
    /// * `maximum` - Inclusive maximum to configure.
    ///
    /// # Returns
    ///
    /// The builder with the described setting applied.
    #[inline]
    #[must_use]
    pub const fn max_sequence_items(mut self, maximum: usize) -> Self {
        self.limits.set_max_sequence_items(maximum);
        self
    }

    /// Sets the maximum number of entries in one map.
    ///
    /// # Parameters
    ///
    /// * `maximum` - Inclusive maximum to configure.
    ///
    /// # Returns
    ///
    /// The builder with the described setting applied.
    #[inline]
    #[must_use]
    pub const fn max_map_entries(mut self, maximum: usize) -> Self {
        self.limits.set_max_map_entries(maximum);
        self
    }

    /// Sets the maximum byte length of one structural key.
    ///
    /// # Parameters
    ///
    /// * `maximum` - Inclusive maximum to configure.
    ///
    /// # Returns
    ///
    /// The builder with the described setting applied.
    #[inline]
    #[must_use]
    pub const fn max_key_bytes(mut self, maximum: usize) -> Self {
        self.limits.set_max_key_bytes(maximum);
        self
    }
}
