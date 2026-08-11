// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
#[cfg(feature = "serde-json")]
mod internal;
#[cfg(feature = "serde-json")]
mod json_decode_tests;
#[cfg(feature = "serde-json")]
mod json_encode_tests;
#[cfg(feature = "serde-json")]
mod json_serde_error_tests;
#[cfg(feature = "serde-json")]
mod json_test_limits_tests;
