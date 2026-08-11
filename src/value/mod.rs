// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================
//! Point limits for common scalar value representations.

#[cfg(feature = "big-decimal")]
mod big_decimal_limits;
#[cfg(feature = "big-integer")]
mod big_integer_limits;
mod string_limits;

#[cfg(feature = "big-decimal")]
pub use big_decimal_limits::BigDecimalLimits;
#[cfg(feature = "big-integer")]
pub use big_integer_limits::BigIntegerLimits;
pub use string_limits::StringLimits;
