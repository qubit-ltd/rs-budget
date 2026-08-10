// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Nested Serde seed used while walking JSON containers.

use serde::de::DeserializeSeed;
use serde::de::Deserializer;

use super::JsonPreflight;
use super::JsonPreflightVisitor;

/// Seed that continues a preflight walk at a nested container depth.
///
/// # Type Parameters
///
/// * `R` - Resource identity reported by budget violations.
pub(in crate::serde) struct JsonPreflightChildSeed<'a, 'b, R> {
    /// Preflight walker shared with the enclosing container visit.
    pub(in crate::serde) preflight: &'a mut JsonPreflight<'b, R>,

    /// Inclusive nesting depth of the child value being visited.
    pub(in crate::serde) depth: usize,
}

impl<'de, 'a, 'b, R> DeserializeSeed<'de> for JsonPreflightChildSeed<'a, 'b, R>
where
    R: Clone,
{
    type Value = ();

    /// Walks one nested JSON value at the seed's configured depth.
    ///
    /// # Parameters
    ///
    /// * `deserializer` - Serde deserializer positioned at the child value.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the nested value was walked without a Serde failure.
    ///
    /// # Errors
    ///
    /// Returns the deserializer error produced by Serde or by a recorded
    /// budget violation.
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(JsonPreflightVisitor {
            preflight: self.preflight,
            depth: self.depth,
        })
    }
}
