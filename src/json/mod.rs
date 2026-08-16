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

mod decode;
mod encode;
mod json_container_kind;
mod json_measurement;
mod json_resource;
mod value;

pub use decode::JsonDecodeAttempt;
pub use decode::JsonDecodeLimits;
pub use decode::JsonDecodeLimitsBuilder;
pub use decode::JsonDecodeSession;
pub use encode::JsonEncodeAttempt;
pub use encode::JsonEncodeLimits;
pub use encode::JsonEncodeLimitsBuilder;
pub use encode::JsonEncodeSession;
pub use json_container_kind::JsonContainerKind;
pub use json_measurement::JsonMeasurement;
pub use json_resource::JsonResource;
pub use value::JsonValueBudget;
pub use value::JsonValueLimits;
pub use value::JsonValueLimitsBuilder;
pub use value::JsonValueTransaction;
