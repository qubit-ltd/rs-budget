// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Budget-aware JSON/Serde adapters.

use std::io::Write;

use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeSeed;
use serde_json::Deserializer as JsonDeserializer;
use serde_json::to_vec;

use super::internal::JsonPreflight;
use crate::JsonBudget;
use crate::JsonSerdeError;

/// Deserializes one JSON slice after charging its complete document.
///
/// # Parameters
///
/// * `input` - Complete JSON document bytes to decode.
/// * `budget` - Mutable JSON budget charged before and during decoding.
///
/// # Returns
///
/// The decoded value when every budget check and Serde decode succeeds.
///
/// # Errors
///
/// Returns [`JsonSerdeError::Budget`] when an input or structural limit is
/// exceeded, or [`JsonSerdeError::Json`] when Serde rejects the document.
///
/// # Type Parameters
///
/// * `T` - Value type decoded from the JSON document.
/// * `R` - Resource identity reported by budget violations.
pub fn from_slice_with_budget<'de, T, R>(
    input: &'de [u8],
    budget: &mut JsonBudget<R, usize>,
) -> Result<T, JsonSerdeError<R>>
where
    T: Deserialize<'de>,
    R: Clone,
{
    preflight(input, budget)?;
    let mut deserializer = JsonDeserializer::from_slice(input);
    let value =
        T::deserialize(&mut deserializer).map_err(JsonSerdeError::Json)?;
    deserializer.end().map_err(JsonSerdeError::Json)?;
    Ok(value)
}

/// Deserializes one JSON slice with a caller-provided Serde seed.
///
/// # Parameters
///
/// * `input` - Complete JSON document bytes to decode.
/// * `seed` - Serde seed that drives typed decoding.
/// * `budget` - Mutable JSON budget charged before and during decoding.
///
/// # Returns
///
/// The seed's decoded value when every budget check and Serde decode
/// succeeds.
///
/// # Errors
///
/// Returns [`JsonSerdeError::Budget`] when an input or structural limit is
/// exceeded, or [`JsonSerdeError::Json`] when Serde rejects the document.
///
/// # Type Parameters
///
/// * `S` - Serde seed that produces the decoded value.
/// * `R` - Resource identity reported by budget violations.
pub fn from_slice_seed_with_budget<'de, S, R>(
    input: &'de [u8],
    seed: S,
    budget: &mut JsonBudget<R, usize>,
) -> Result<S::Value, JsonSerdeError<R>>
where
    S: DeserializeSeed<'de>,
    R: Clone,
{
    preflight(input, budget)?;
    let mut deserializer = JsonDeserializer::from_slice(input);
    let value = seed
        .deserialize(&mut deserializer)
        .map_err(JsonSerdeError::Json)?;
    deserializer.end().map_err(JsonSerdeError::Json)?;
    Ok(value)
}

/// Serializes one value to a compact JSON vector after charging its output.
///
/// # Parameters
///
/// * `value` - Value serialized into compact JSON.
/// * `budget` - Mutable JSON budget charged for output and structure.
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
pub fn to_vec_with_budget<T, R>(
    value: &T,
    budget: &mut JsonBudget<R, usize>,
) -> Result<Vec<u8>, JsonSerdeError<R>>
where
    T: Serialize + ?Sized,
    R: Clone,
{
    let bytes = to_vec(value).map_err(JsonSerdeError::Json)?;
    budget
        .check_output_bytes(bytes.len())
        .map_err(JsonSerdeError::Budget)?;
    preflight_without_input_limit(&bytes, budget)?;
    Ok(bytes)
}

/// Serializes one value and writes it only after budget checks pass.
///
/// # Parameters
///
/// * `writer` - Destination that receives the compact JSON bytes.
/// * `value` - Value serialized into compact JSON.
/// * `budget` - Mutable JSON budget charged for output and structure.
///
/// # Returns
///
/// `Ok(())` when serialization, budget checks, and the write all succeed.
///
/// # Errors
///
/// Returns [`JsonSerdeError::Json`] or [`JsonSerdeError::Budget`] from
/// [`to_vec_with_budget`], or [`JsonSerdeError::Io`] when the writer fails.
///
/// # Type Parameters
///
/// * `W` - Writer that accepts the serialized JSON bytes.
/// * `T` - Value type serialized to JSON.
/// * `R` - Resource identity reported by budget violations.
pub fn to_writer_with_budget<W, T, R>(
    mut writer: W,
    value: &T,
    budget: &mut JsonBudget<R, usize>,
) -> Result<(), JsonSerdeError<R>>
where
    W: Write,
    T: Serialize + ?Sized,
    R: Clone,
{
    let bytes = to_vec_with_budget(value, budget)?;
    writer.write_all(&bytes).map_err(JsonSerdeError::Io)
}

/// Charges input size and walks the document before typed decoding.
///
/// # Parameters
///
/// * `input` - Complete JSON document bytes to inspect.
/// * `budget` - Mutable JSON budget charged for input and structure.
///
/// # Returns
///
/// `Ok(())` when every input and structural budget check succeeds.
///
/// # Errors
///
/// Returns [`JsonSerdeError::Budget`] when a limit is exceeded, or
/// [`JsonSerdeError::Json`] when Serde rejects the document during preflight.
///
/// # Type Parameters
///
/// * `R` - Resource identity reported by budget violations.
fn preflight<R>(
    input: &[u8],
    budget: &mut JsonBudget<R, usize>,
) -> Result<(), JsonSerdeError<R>>
where
    R: Clone,
{
    budget
        .check_input_bytes(input.len())
        .map_err(JsonSerdeError::Budget)?;
    preflight_without_input_limit(input, budget)
}

/// Walks one JSON document and charges structural limits only.
///
/// # Parameters
///
/// * `input` - Complete JSON document bytes to inspect.
/// * `budget` - Mutable JSON budget charged for structure and value sizes.
///
/// # Returns
///
/// `Ok(())` when every structural budget check succeeds.
///
/// # Errors
///
/// Returns [`JsonSerdeError::Budget`] when a limit is exceeded, or
/// [`JsonSerdeError::Json`] when Serde rejects the document during preflight.
///
/// # Type Parameters
///
/// * `R` - Resource identity reported by budget violations.
fn preflight_without_input_limit<R>(
    input: &[u8],
    budget: &mut JsonBudget<R, usize>,
) -> Result<(), JsonSerdeError<R>>
where
    R: Clone,
{
    let mut deserializer = JsonDeserializer::from_slice(input);
    let (result, violation) = {
        let mut visitor = JsonPreflight::new(budget);
        let result = (&mut visitor).deserialize(&mut deserializer);
        (result, visitor.take_violation())
    };
    if let Some(error) = violation {
        return Err(JsonSerdeError::Budget(error));
    }
    if let Err(error) = result {
        return Err(JsonSerdeError::Json(error));
    }
    deserializer.end().map_err(JsonSerdeError::Json)
}
