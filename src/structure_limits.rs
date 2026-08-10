// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines optional structural input limits.

use crate::ResourceLimit;
use crate::StructureBudget;
use crate::StructureResource;

/// Optional limits for processing nested structural data.
///
/// Each configured maximum is inclusive. Depth, sequence items, and map
/// entries are checked for each individual value, while nodes are consumed
/// cumulatively by each budget session created with [`Self::budget`].
#[must_use]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct StructureLimits {
    /// Optional inclusive maximum nesting depth.
    pub(crate) max_depth: Option<ResourceLimit<StructureResource, usize>>,

    /// Optional cumulative maximum number of processed nodes.
    pub(crate) max_nodes: Option<ResourceLimit<StructureResource, usize>>,

    /// Optional inclusive maximum number of items in one sequence.
    pub(crate) max_sequence_items: Option<ResourceLimit<StructureResource, usize>>,

    /// Optional inclusive maximum number of entries in one map.
    pub(crate) max_map_entries: Option<ResourceLimit<StructureResource, usize>>,
}

impl StructureLimits {
    /// Creates a configuration with every structural limit unconfigured.
    ///
    /// # Returns
    ///
    /// Limits whose checks always pass and whose budget sessions do not track
    /// nodes until a corresponding `with_max_*` method is used.
    #[inline]
    pub const fn new() -> Self {
        Self {
            max_depth: None,
            max_nodes: None,
            max_sequence_items: None,
            max_map_entries: None,
        }
    }

    /// Configures the inclusive maximum nesting depth.
    ///
    /// # Parameters
    ///
    /// * `maximum` - Largest depth accepted by a budget session.
    ///
    /// # Returns
    ///
    /// Updated limits that reject depths greater than `maximum`.
    #[inline]
    pub const fn with_max_depth(mut self, maximum: usize) -> Self {
        self.max_depth = Some(ResourceLimit::new(StructureResource::Depth, maximum));
        self
    }

    /// Configures the cumulative maximum number of processed nodes.
    ///
    /// # Parameters
    ///
    /// * `maximum` - Total nodes accepted by each new budget session.
    ///
    /// # Returns
    ///
    /// Updated limits whose new budget sessions track at most `maximum` nodes.
    #[inline]
    pub const fn with_max_nodes(mut self, maximum: usize) -> Self {
        self.max_nodes = Some(ResourceLimit::new(StructureResource::Nodes, maximum));
        self
    }

    /// Configures the inclusive maximum item count for one sequence.
    ///
    /// # Parameters
    ///
    /// * `maximum` - Largest sequence item count accepted by a point check.
    ///
    /// # Returns
    ///
    /// Updated limits that reject individual sequences larger than `maximum`.
    #[inline]
    pub const fn with_max_sequence_items(mut self, maximum: usize) -> Self {
        self.max_sequence_items = Some(ResourceLimit::new(
            StructureResource::SequenceItems,
            maximum,
        ));
        self
    }

    /// Configures the inclusive maximum entry count for one map.
    ///
    /// # Parameters
    ///
    /// * `maximum` - Largest map entry count accepted by a point check.
    ///
    /// # Returns
    ///
    /// Updated limits that reject individual maps larger than `maximum`.
    #[inline]
    pub const fn with_max_map_entries(mut self, maximum: usize) -> Self {
        self.max_map_entries = Some(ResourceLimit::new(StructureResource::MapEntries, maximum));
        self
    }

    /// Creates an independent structural budget session from these limits.
    ///
    /// # Returns
    ///
    /// A session with fresh node capacity. Point limits remain immutable and are
    /// copied into the session configuration.
    #[inline]
    pub fn budget(&self) -> StructureBudget {
        StructureBudget::new(*self)
    }
}
