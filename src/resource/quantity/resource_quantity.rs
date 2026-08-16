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

use super::internal::ResourceQuantitySealed;
use crate::resource::QuantityConversionError;
use crate::resource::QuantityMeasurement;

/// An exact, non-negative quantity accepted by resource budgets and pools.
///
/// Implementations are limited to Rust's unsigned integer types. This excludes
/// signed values, floating-point `NaN`/infinity, and rounding-based arithmetic
/// from the accounting invariants.
pub trait ResourceQuantity:
    ResourceQuantitySealed
    + Copy
    + Debug
    + Display
    + Eq
    + Ord
    + Add<Output = Self>
    + Sub<Output = Self>
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
    #[must_use]
    fn checked_add(self, other: Self) -> Option<Self>;

    /// Converts one Rust-native length into this resource quantity.
    ///
    /// # Parameters
    ///
    /// * `value` - Length reported by a string, collection, or Serde API.
    ///
    /// # Returns
    ///
    /// Returns the exact quantity when it fits this type.
    ///
    /// # Errors
    ///
    /// Returns [`QuantityConversionError`] when `value` cannot be represented
    /// without truncation.
    fn try_from_usize(value: usize) -> Result<Self, QuantityConversionError>;

    /// Converts one stable 64-bit measurement into this resource quantity.
    ///
    /// # Parameters
    ///
    /// * `value` - Measurement reported by an API using `u64`.
    ///
    /// # Returns
    ///
    /// Returns the exact quantity when it fits this type.
    ///
    /// # Errors
    ///
    /// Returns [`QuantityConversionError`] when `value` cannot be represented
    /// without truncation.
    fn try_from_u64(value: u64) -> Result<Self, QuantityConversionError>;
}

macro_rules! impl_resource_quantity {
    ($($quantity:ty),+ $(,)?) => {
        $(
            impl ResourceQuantitySealed for $quantity {}

            impl ResourceQuantity for $quantity {
                const ZERO: Self = 0;
                const ONE: Self = 1;

                #[inline]
                fn checked_add(self, other: Self) -> Option<Self> {
                    Self::checked_add(self, other)
                }

                #[inline]
                fn try_from_usize(value: usize) -> Result<Self, QuantityConversionError> {
                    <$quantity>::try_from(value).map_err(|_| {
                        QuantityConversionError::new(
                            QuantityMeasurement::Usize(value),
                            stringify!($quantity),
                        )
                    })
                }

                #[inline]
                fn try_from_u64(value: u64) -> Result<Self, QuantityConversionError> {
                    <$quantity>::try_from(value).map_err(|_| {
                        QuantityConversionError::new(
                            QuantityMeasurement::U64(value),
                            stringify!($quantity),
                        )
                    })
                }
            }
        )+
    };
}

impl_resource_quantity!(u8, u16, u32, u64, u128, usize);
