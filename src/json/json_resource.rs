// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines resource identities for JSON processing limits.

/// A JSON quantity constrained while processing one JSON input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JsonResource {
    /// The total byte length of one complete JSON input.
    InputBytes,

    /// The root-inclusive nesting depth of the current JSON value.
    Depth,

    /// The cumulative number of JSON nodes processed in one session.
    Nodes,

    /// The number of items in one JSON array.
    SequenceItems,

    /// The number of entries in one JSON object.
    MapEntries,

    /// The byte length of one JSON string value.
    StringBytes,

    /// The byte length of one JSON number representation.
    NumberBytes,
}
