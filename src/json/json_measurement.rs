// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines native measurements collected while traversing JSON values.

/// Native dimensions measured for one JSON value or object key.
///
/// # Examples
///
/// ```
/// use qubit_budget::json::JsonMeasurement;
///
/// let measurement = JsonMeasurement::String { depth: 2, bytes: 5 };
/// assert!(matches!(measurement, JsonMeasurement::String { bytes: 5, .. }));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JsonMeasurement {
    /// A null value measured by its root-inclusive nesting depth.
    Null {
        /// Root-inclusive nesting depth of the null value.
        depth: usize,
    },
    /// A boolean value measured by its root-inclusive nesting depth.
    Boolean {
        /// Root-inclusive nesting depth of the boolean value.
        depth: usize,
    },
    /// A string value measured by its nesting depth and UTF-8 byte length.
    String {
        /// Root-inclusive nesting depth of the string value.
        depth: usize,
        /// UTF-8 byte length of the string value.
        bytes: usize,
    },
    /// A number value measured by its nesting depth and representation length.
    Number {
        /// Root-inclusive nesting depth of the number value.
        depth: usize,
        /// Byte length of the number representation.
        bytes: usize,
    },
    /// An array measured by its nesting depth and item count.
    Array {
        /// Root-inclusive nesting depth of the array.
        depth: usize,
        /// Number of direct items in the array.
        items: usize,
    },
    /// An object measured by its nesting depth and entry count.
    Object {
        /// Root-inclusive nesting depth of the object.
        depth: usize,
        /// Number of direct entries in the object.
        entries: usize,
    },
    /// An object key measured by its UTF-8 byte length.
    Key {
        /// UTF-8 byte length of the object key.
        bytes: usize,
    },
}
