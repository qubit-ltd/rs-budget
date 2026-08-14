// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Private storage types for JSON budget sessions.

mod decode_storage;
mod encode_storage;
mod json_value_state;
mod prepared_json_admission;

pub(super) use decode_storage::DecodeStorage;
pub(super) use encode_storage::EncodeStorage;
pub(in crate::json) use json_value_state::JsonValueState;
pub(in crate::json) use prepared_json_admission::PreparedJsonAdmission;
