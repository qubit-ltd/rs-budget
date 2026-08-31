// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Point limits for common scalar value representations.

#[cfg(feature = "big-decimal")]
mod big_decimal_limits;
#[cfg(feature = "big-decimal")]
mod big_decimal_limits_builder;
#[cfg(feature = "big-integer")]
mod big_integer_limits;
#[cfg(feature = "big-integer")]
mod big_integer_limits_builder;
mod string_limits;
mod string_limits_builder;

#[cfg(feature = "big-decimal")]
#[cfg_attr(docsrs, doc(cfg(feature = "big-decimal")))]
pub use big_decimal_limits::BigDecimalLimits;
#[cfg(feature = "big-decimal")]
#[cfg_attr(docsrs, doc(cfg(feature = "big-decimal")))]
pub use big_decimal_limits_builder::BigDecimalLimitsBuilder;
#[cfg(feature = "big-integer")]
#[cfg_attr(docsrs, doc(cfg(feature = "big-integer")))]
pub use big_integer_limits::BigIntegerLimits;
#[cfg(feature = "big-integer")]
#[cfg_attr(docsrs, doc(cfg(feature = "big-integer")))]
pub use big_integer_limits_builder::BigIntegerLimitsBuilder;
pub use string_limits::StringLimits;
pub use string_limits_builder::StringLimitsBuilder;
