// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Private state used by JSON value accounting.

mod json_value_state;
mod prepared_json_admission;

pub(in crate::json) use json_value_state::JsonValueState;
pub(in crate::json) use prepared_json_admission::PreparedJsonAdmission;
