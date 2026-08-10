// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines optional JSON processing limits.

use crate::ResourceLimit;
use crate::StructureLimits;
use crate::json::JsonBudget;
use crate::json::JsonResource;

/// Optional limits for processing one JSON input.
///
/// Each configured maximum is inclusive. Input, depth, container, string, and
/// number checks apply to one observed value. Node charges accumulate in each
/// budget session created with [`Self::budget`].
#[must_use]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct JsonLimits {
    /// Optional inclusive maximum byte length of a complete input.
    pub(crate) max_input_bytes: Option<ResourceLimit<JsonResource, usize>>,

    /// Optional inclusive root-inclusive nesting depth.
    pub(crate) max_depth: Option<ResourceLimit<JsonResource, usize>>,

    /// Optional cumulative maximum number of processed nodes.
    pub(crate) max_nodes: Option<ResourceLimit<JsonResource, usize>>,

    /// Optional inclusive maximum number of items in one array.
    pub(crate) max_sequence_items: Option<ResourceLimit<JsonResource, usize>>,

    /// Optional inclusive maximum number of entries in one object.
    pub(crate) max_map_entries: Option<ResourceLimit<JsonResource, usize>>,

    /// Optional inclusive maximum byte length of one string.
    pub(crate) max_string_bytes: Option<ResourceLimit<JsonResource, usize>>,

    /// Optional inclusive maximum byte length of one number representation.
    pub(crate) max_number_bytes: Option<ResourceLimit<JsonResource, usize>>,
}

impl JsonLimits {
    /// Creates a configuration with every JSON limit unconfigured.
    ///
    /// # Returns
    ///
    /// Limits whose checks always pass and whose budget sessions do not track
    /// nodes until a corresponding `with_max_*` method is used.
    #[inline]
    pub const fn new() -> Self {
        Self {
            max_input_bytes: None,
            max_depth: None,
            max_nodes: None,
            max_sequence_items: None,
            max_map_entries: None,
            max_string_bytes: None,
            max_number_bytes: None,
        }
    }

    /// Configures the inclusive maximum byte length of a complete JSON input.
    ///
    /// # Parameters
    ///
    /// * `maximum` - Largest complete input byte length accepted by a budget.
    ///
    /// # Returns
    ///
    /// Updated limits that reject complete inputs larger than `maximum`.
    #[inline]
    pub const fn with_max_input_bytes(mut self, maximum: usize) -> Self {
        self.max_input_bytes = Some(ResourceLimit::new(JsonResource::InputBytes, maximum));
        self
    }

    /// Configures the inclusive root-inclusive maximum nesting depth.
    ///
    /// # Parameters
    ///
    /// * `maximum` - Largest root-inclusive depth accepted by a budget.
    ///
    /// # Returns
    ///
    /// Updated limits that reject depths greater than `maximum`.
    #[inline]
    pub const fn with_max_depth(mut self, maximum: usize) -> Self {
        self.max_depth = Some(ResourceLimit::new(JsonResource::Depth, maximum));
        self
    }

    /// Configures the cumulative maximum number of processed JSON nodes.
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
        self.max_nodes = Some(ResourceLimit::new(JsonResource::Nodes, maximum));
        self
    }

    /// Configures the inclusive maximum item count for one JSON array.
    ///
    /// # Parameters
    ///
    /// * `maximum` - Largest item count accepted by an array point check.
    ///
    /// # Returns
    ///
    /// Updated limits that reject individual arrays larger than `maximum`.
    #[inline]
    pub const fn with_max_sequence_items(mut self, maximum: usize) -> Self {
        self.max_sequence_items = Some(ResourceLimit::new(JsonResource::SequenceItems, maximum));
        self
    }

    /// Configures the inclusive maximum entry count for one JSON object.
    ///
    /// # Parameters
    ///
    /// * `maximum` - Largest entry count accepted by an object point check.
    ///
    /// # Returns
    ///
    /// Updated limits that reject individual objects larger than `maximum`.
    #[inline]
    pub const fn with_max_map_entries(mut self, maximum: usize) -> Self {
        self.max_map_entries = Some(ResourceLimit::new(JsonResource::MapEntries, maximum));
        self
    }

    /// Configures the inclusive maximum byte length for one JSON string.
    ///
    /// # Parameters
    ///
    /// * `maximum` - Largest string byte length accepted by a point check.
    ///
    /// # Returns
    ///
    /// Updated limits that reject individual strings larger than `maximum`.
    #[inline]
    pub const fn with_max_string_bytes(mut self, maximum: usize) -> Self {
        self.max_string_bytes = Some(ResourceLimit::new(JsonResource::StringBytes, maximum));
        self
    }

    /// Configures the inclusive maximum byte length for one JSON number.
    ///
    /// # Parameters
    ///
    /// * `maximum` - Largest number representation byte length accepted by a
    ///   point check.
    ///
    /// # Returns
    ///
    /// Updated limits that reject individual numbers larger than `maximum`.
    #[inline]
    pub const fn with_max_number_bytes(mut self, maximum: usize) -> Self {
        self.max_number_bytes = Some(ResourceLimit::new(JsonResource::NumberBytes, maximum));
        self
    }

    /// Replaces the structural limits with the supplied configuration.
    #[inline]
    pub const fn with_structure_limits(mut self, limits: StructureLimits) -> Self {
        self.max_depth = match limits.max_depth() {
            Some(maximum) => Some(ResourceLimit::new(JsonResource::Depth, maximum)),
            None => None,
        };
        self.max_nodes = match limits.max_nodes() {
            Some(maximum) => Some(ResourceLimit::new(JsonResource::Nodes, maximum)),
            None => None,
        };
        self.max_sequence_items = match limits.max_sequence_items() {
            Some(maximum) => Some(ResourceLimit::new(JsonResource::SequenceItems, maximum)),
            None => None,
        };
        self.max_map_entries = match limits.max_map_entries() {
            Some(maximum) => Some(ResourceLimit::new(JsonResource::MapEntries, maximum)),
            None => None,
        };
        self
    }

    /// Returns the structural limits represented by this JSON configuration.
    #[inline(always)]
    pub const fn structure_limits(&self) -> StructureLimits {
        let mut limits = StructureLimits::new();
        if let Some(maximum) = self.max_depth() {
            limits = limits.with_max_depth(maximum);
        }
        if let Some(maximum) = self.max_nodes() {
            limits = limits.with_max_nodes(maximum);
        }
        if let Some(maximum) = self.max_sequence_items() {
            limits = limits.with_max_sequence_items(maximum);
        }
        if let Some(maximum) = self.max_map_entries() {
            limits = limits.with_max_map_entries(maximum);
        }
        limits
    }

    /// Returns the configured maximum root-inclusive nesting depth.
    #[must_use]
    #[inline(always)]
    pub const fn max_depth(&self) -> Option<usize> {
        match self.max_depth {
            Some(limit) => Some(limit.maximum()),
            None => None,
        }
    }

    /// Returns the configured maximum number of processed nodes.
    #[must_use]
    #[inline(always)]
    pub const fn max_nodes(&self) -> Option<usize> {
        match self.max_nodes {
            Some(limit) => Some(limit.maximum()),
            None => None,
        }
    }

    /// Returns the configured maximum number of items in one array.
    #[must_use]
    #[inline(always)]
    pub const fn max_sequence_items(&self) -> Option<usize> {
        match self.max_sequence_items {
            Some(limit) => Some(limit.maximum()),
            None => None,
        }
    }

    /// Returns the configured maximum number of entries in one object.
    #[must_use]
    #[inline(always)]
    pub const fn max_map_entries(&self) -> Option<usize> {
        match self.max_map_entries {
            Some(limit) => Some(limit.maximum()),
            None => None,
        }
    }

    /// Creates an independent JSON budget session from these limits.
    ///
    /// # Returns
    ///
    /// A session with fresh node capacity. Point limits remain immutable and
    /// are copied into the session configuration.
    #[inline]
    pub fn budget(&self) -> JsonBudget {
        JsonBudget::new(*self)
    }
}
