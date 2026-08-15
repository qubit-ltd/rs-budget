// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the JSON container dimensions checked during traversal.

/// Identifies the point-limited count of a JSON container.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum JsonContainerKind {
    /// The item count of one JSON array.
    Sequence,

    /// The entry count of one JSON object.
    Map,
}
