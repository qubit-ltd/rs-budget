// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Budget-aware JSON encoding APIs.

use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

use serde::Serialize;
use serde_json::Serializer as JsonSerializer;

use super::internal::JsonEncodeSerializer;
use super::internal::JsonOutputAccounting;
use super::internal::JsonOutputBuffer;
use crate::JsonEncodeSession;
use crate::JsonSerdeError;

/// Serializes one value to a compact JSON vector while charging its output.
///
/// # Parameters
///
/// * `value` - Value serialized into compact JSON.
/// * `session` - Mutable JSON session charged before delegation and output
///   growth.
///
/// # Returns
///
/// Compact JSON bytes when serialization and every budget check succeed.
///
/// # Errors
///
/// Returns [`JsonSerdeError::Json`] when serialization fails, or
/// [`JsonSerdeError::Budget`] when an output or structural limit is exceeded.
///
/// # Type Parameters
///
/// * `T` - Value type serialized to JSON.
/// * `R` - Resource identity reported by budget violations.
pub fn encode_to_vec<T, R>(
    value: &T,
    session: &mut JsonEncodeSession<R, u64>,
) -> Result<Vec<u8>, JsonSerdeError<R>>
where
    T: Serialize + ?Sized,
    R: Clone,
{
    let (output_budget, value_budget) = session.split_mut();
    let accounting = Rc::new(RefCell::new(JsonOutputAccounting::new(output_budget)));
    let mut output = JsonOutputBuffer::new(Rc::clone(&accounting));
    let result = {
        let mut inner = JsonSerializer::new(&mut output);
        value.serialize(JsonEncodeSerializer::new(
            &mut inner,
            value_budget,
            accounting,
        ))
    };
    output.into_result(result)
}

/// Serializes one value and writes it only after budget checks pass.
///
/// Serialization is transactional with respect to budget and Serde failures:
/// the destination is not touched until the complete buffered document is
/// accepted. A failure during the final [`Write::write_all`] call may leave the
/// destination with a partial document because [`Write`] has no rollback API.
///
/// # Parameters
///
/// * `writer` - Destination that receives the compact JSON bytes.
/// * `value` - Value serialized into compact JSON.
/// * `session` - Mutable JSON session charged for output and value resources.
///
/// # Returns
///
/// `Ok(())` when serialization, budget checks, and the write all succeed.
///
/// # Errors
///
/// Returns [`JsonSerdeError::Json`] or [`JsonSerdeError::Budget`] from
/// [`encode_to_vec`], or [`JsonSerdeError::Io`] when the writer fails.
///
/// # Type Parameters
///
/// * `W` - Writer that accepts the serialized JSON bytes.
/// * `T` - Value type serialized to JSON.
/// * `R` - Resource identity reported by budget violations.
pub fn encode_to_writer<W, T, R>(
    mut writer: W,
    value: &T,
    session: &mut JsonEncodeSession<R, u64>,
) -> Result<(), JsonSerdeError<R>>
where
    W: Write,
    T: Serialize + ?Sized,
    R: Clone,
{
    let bytes = encode_to_vec(value, session)?;
    writer.write_all(&bytes).map_err(JsonSerdeError::Io)
}
