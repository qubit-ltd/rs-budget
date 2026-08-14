// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines dependency-free resource limits and sessions for JSON processing.
//!
//! Value budgets admit one node at a time. A rejected node leaves its own
//! structural and payload counters unchanged, while earlier admitted nodes
//! remain charged for the lifetime of the session. Decode sessions likewise
//! consume input bytes before parsing and do not roll those charges back when
//! later syntax or typed decoding fails.
//!
//! Encode sessions keep output accounting transactional at the document
//! boundary: output is charged to a temporary snapshot and committed only
//! after serialization succeeds. This output guarantee does not roll back
//! structural value charges already accepted during serialization.

mod internal;
mod json_decode_limits;
mod json_decode_session;
mod json_encode_limits;
mod json_encode_session;
mod json_resource;
mod json_value_budget;
mod json_value_limits;

pub use json_decode_limits::JsonDecodeLimits;
pub use json_decode_session::JsonDecodeSession;
pub use json_encode_limits::JsonEncodeLimits;
pub use json_encode_session::JsonEncodeSession;
pub use json_resource::JsonResource;
pub use json_value_budget::JsonValueBudget;
pub use json_value_limits::JsonValueLimits;
