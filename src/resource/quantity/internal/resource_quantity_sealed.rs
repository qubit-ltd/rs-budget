// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Seals the set of supported resource quantity implementations.

/// Private marker implemented only by the crate's supported unsigned integers.
pub trait ResourceQuantitySealed {
    /// Converts this supported quantity to `u128` without losing information.
    #[allow(clippy::wrong_self_convention)] // The sealed conversion consumes the quantity.
    fn as_u128(self) -> u128;
}
