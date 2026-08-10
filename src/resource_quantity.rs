// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the exact unsigned quantities accepted by resource accounting.

use std::fmt::Debug;
use std::fmt::Display;
use std::ops::Add;
use std::ops::Sub;

mod private {
    /// Seals [`super::ResourceQuantity`] to the supported unsigned integers.
    pub trait Sealed {}
}

/// An exact, non-negative quantity accepted by resource budgets and pools.
///
/// Implementations are limited to Rust's unsigned integer types. This excludes
/// signed values, floating-point `NaN`/infinity, and rounding-based arithmetic
/// from the accounting invariants.
pub trait ResourceQuantity:
    private::Sealed + Copy + Debug + Display + Eq + Ord + Add<Output = Self> + Sub<Output = Self>
{
    /// The additive identity for this quantity type.
    const ZERO: Self;

    /// The unit quantity for this quantity type.
    const ONE: Self;

    /// Adds two quantities when their sum is representable.
    ///
    /// # Parameters
    ///
    /// * `other` - Quantity to add.
    ///
    /// # Returns
    ///
    /// `Some(sum)` when the sum fits in the quantity type, or `None` on
    /// overflow.
    fn checked_add(self, other: Self) -> Option<Self>;
}

macro_rules! impl_resource_quantity {
    ($($quantity:ty),+ $(,)?) => {
        $(
            impl private::Sealed for $quantity {}

            impl ResourceQuantity for $quantity {
                const ZERO: Self = 0;
                const ONE: Self = 1;

                #[inline]
                fn checked_add(self, other: Self) -> Option<Self> {
                    Self::checked_add(self, other)
                }
            }
        )+
    };
}

impl_resource_quantity!(u8, u16, u32, u64, u128, usize);
