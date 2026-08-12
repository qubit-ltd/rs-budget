// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Compatibility classification for serde_json private serializer shapes.

/// Private serde_json token used by arbitrary-precision JSON numbers.
const JSON_NUMBER_TOKEN: &str =
    concat!("$", "serde_json", ":", ":private::Number");

/// Private serde_json token used by raw JSON fragments.
const JSON_RAW_VALUE_TOKEN: &str =
    concat!("$", "serde_json", ":", ":private::RawValue");

/// Tests whether a serializer struct name denotes serde_json's number shape.
#[inline(always)]
pub(super) fn is_number(name: &str) -> bool {
    name == JSON_NUMBER_TOKEN
}

/// Tests whether a serializer struct name denotes serde_json's raw-value shape.
#[inline(always)]
pub(super) fn is_raw_value(name: &str) -> bool {
    name == JSON_RAW_VALUE_TOKEN
}
