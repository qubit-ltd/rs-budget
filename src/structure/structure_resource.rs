// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines resource identities for structural input limits.

/// A structural quantity constrained while processing nested data.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StructureResource {
    /// The nesting depth of the current value.
    Depth,

    /// The cumulative number of nodes processed in one session.
    Nodes,

    /// The number of items in one sequence value.
    SequenceItems,

    /// The number of entries in one map value.
    MapEntries,

    /// The byte length of one structural key.
    KeyBytes,
}
