// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! JSON encoding limits and session accounting.

mod internal;
mod json_encode_attempt;
mod json_encode_limits;
mod json_encode_limits_builder;
mod json_encode_session;

pub use json_encode_attempt::JsonEncodeAttempt;
pub use json_encode_limits::JsonEncodeLimits;
pub use json_encode_limits_builder::JsonEncodeLimitsBuilder;
pub use json_encode_session::JsonEncodeSession;
