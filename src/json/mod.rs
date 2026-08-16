// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines dependency-free resource limits and sessions for JSON processing.
//!
//! Value budgets admit one complete value through a transaction. Staged usage
//! becomes committed only when that transaction succeeds, so a rejected value
//! or unwinding rollback cannot consume structural or payload capacity.
//!
//! A JSON encoder that buffers a complete `Vec<u8>` can charge output only
//! after serialization succeeds. Writer-oriented encoders instead charge every
//! accepted output prefix immediately, so those charges remain after a later
//! error or panic.
//!
//! Decode and encode attempts both retain their immediate I/O charges. They
//! publish staged value accounting only through `commit`; dropping an attempt,
//! including while unwinding, rolls the value state back.

mod internal;
mod json_container_kind;
mod json_decode_attempt;
mod json_decode_limits;
mod json_decode_session;
mod json_encode_attempt;
mod json_encode_limits;
mod json_encode_session;
mod json_measurement;
mod json_resource;
mod json_value_budget;
mod json_value_limits;
mod json_value_transaction;

pub use json_container_kind::JsonContainerKind;
pub use json_decode_attempt::JsonDecodeAttempt;
pub use json_decode_limits::JsonDecodeLimits;
pub use json_decode_limits::JsonDecodeLimitsBuilder;
pub use json_decode_session::JsonDecodeSession;
pub use json_encode_attempt::JsonEncodeAttempt;
pub use json_encode_limits::JsonEncodeLimits;
pub use json_encode_limits::JsonEncodeLimitsBuilder;
pub use json_encode_session::JsonEncodeSession;
pub use json_measurement::JsonMeasurement;
pub use json_resource::JsonResource;
pub use json_value_budget::JsonValueBudget;
pub use json_value_limits::JsonValueLimits;
pub use json_value_limits::JsonValueLimitsBuilder;
pub use json_value_transaction::JsonValueTransaction;
