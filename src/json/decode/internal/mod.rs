// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Private storage for JSON decoding sessions.

mod decode_storage;
mod input_accounting;

pub(super) use decode_storage::DecodeStorage;
pub(super) use input_accounting::consume_bytes;
