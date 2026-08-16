// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! JSON decoding limits and session accounting.

mod internal;
mod json_decode_attempt;
mod json_decode_limits;
mod json_decode_limits_builder;
mod json_decode_session;

pub use json_decode_attempt::JsonDecodeAttempt;
pub use json_decode_limits::JsonDecodeLimits;
pub use json_decode_limits_builder::JsonDecodeLimitsBuilder;
pub use json_decode_session::JsonDecodeSession;
