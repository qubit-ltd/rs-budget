// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================
//! Budget-aware JSON/Serde adapters.

use std::fmt;
use std::io::Write;

use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeSeed;
use serde::de::Deserializer;
use serde::de::Error as DeError;
use serde::de::MapAccess;
use serde::de::SeqAccess;
use serde::de::Visitor;

use crate::BudgetError;
use crate::JsonBudget;
use crate::JsonSerdeError;

/// Deserializes one JSON slice after charging its complete document.
pub fn from_slice_with_budget<'de, T, R>(
    input: &'de [u8],
    budget: &mut JsonBudget<R, usize>,
) -> Result<T, JsonSerdeError<R>>
where
    T: Deserialize<'de>,
    R: Clone,
{
    preflight(input, budget)?;
    let mut deserializer = ::serde_json::Deserializer::from_slice(input);
    let value =
        T::deserialize(&mut deserializer).map_err(JsonSerdeError::Json)?;
    deserializer.end().map_err(JsonSerdeError::Json)?;
    Ok(value)
}

/// Deserializes one JSON slice with a caller-provided Serde seed.
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
    let mut deserializer = ::serde_json::Deserializer::from_slice(input);
    let value = seed
        .deserialize(&mut deserializer)
        .map_err(JsonSerdeError::Json)?;
    deserializer.end().map_err(JsonSerdeError::Json)?;
    Ok(value)
}

/// Serializes one value to a compact JSON vector after charging its output.
pub fn to_vec_with_budget<T, R>(
    value: &T,
    budget: &mut JsonBudget<R, usize>,
) -> Result<Vec<u8>, JsonSerdeError<R>>
where
    T: Serialize + ?Sized,
    R: Clone,
{
    let bytes = ::serde_json::to_vec(value).map_err(JsonSerdeError::Json)?;
    budget
        .check_output_bytes(bytes.len())
        .map_err(JsonSerdeError::Budget)?;
    preflight_without_input_limit(&bytes, budget)?;
    Ok(bytes)
}

/// Serializes one value and writes it only after budget checks pass.
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

fn preflight_without_input_limit<R>(
    input: &[u8],
    budget: &mut JsonBudget<R, usize>,
) -> Result<(), JsonSerdeError<R>>
where
    R: Clone,
{
    let mut deserializer = ::serde_json::Deserializer::from_slice(input);
    let (result, violation) = {
        let mut visitor = JsonPreflight::new(budget);
        let result = (&mut visitor).deserialize(&mut deserializer);
        (result, visitor.violation.take())
    };
    if let Some(error) = violation {
        return Err(JsonSerdeError::Budget(error));
    }
    if let Err(error) = result {
        return Err(JsonSerdeError::Json(error));
    }
    deserializer.end().map_err(JsonSerdeError::Json)
}

const JSON_NUMBER_TOKEN: &str = "$serde_json::private::Number";

struct JsonPreflight<'a, R> {
    budget: &'a mut JsonBudget<R, usize>,
    violation: Option<BudgetError<R, usize>>,
}

impl<'a, R> JsonPreflight<'a, R>
where
    R: Clone,
{
    fn new(budget: &'a mut JsonBudget<R, usize>) -> Self {
        Self {
            budget,
            violation: None,
        }
    }

    fn record<E>(
        &mut self,
        result: Result<(), BudgetError<R, usize>>,
    ) -> Result<(), E>
    where
        E: DeError,
    {
        result.map_err(|error| {
            self.violation = Some(error);
            E::custom("JSON resource budget exceeded")
        })
    }

    fn node<E>(&mut self, depth: usize) -> Result<(), E>
    where
        E: DeError,
    {
        let result = self.budget.enter_node(depth);
        self.record(result)
    }

    fn sequence_items<E>(&mut self, items: usize) -> Result<(), E>
    where
        E: DeError,
    {
        self.record(self.budget.check_sequence_items(items))
    }

    fn map_entries<E>(&mut self, entries: usize) -> Result<(), E>
    where
        E: DeError,
    {
        self.record(self.budget.check_map_entries(entries))
    }

    fn key<E>(&mut self, bytes: usize) -> Result<(), E>
    where
        E: DeError,
    {
        self.record(self.budget.check_key_bytes(bytes))
    }

    fn string<E>(&mut self, bytes: usize) -> Result<(), E>
    where
        E: DeError,
    {
        self.record(self.budget.check_string_bytes(bytes))
    }

    fn number<E>(&mut self, bytes: usize) -> Result<(), E>
    where
        E: DeError,
    {
        self.record(self.budget.check_number_bytes(bytes))
    }
}

impl<'de, 'a, 'b, R> DeserializeSeed<'de> for &'a mut JsonPreflight<'b, R>
where
    R: Clone,
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(JsonPreflightVisitor {
            preflight: self,
            depth: 1,
        })
    }
}

struct JsonPreflightVisitor<'a, 'b, R> {
    preflight: &'a mut JsonPreflight<'b, R>,
    depth: usize,
}

impl<R> JsonPreflightVisitor<'_, '_, R>
where
    R: Clone,
{
    fn scalar<E>(&mut self) -> Result<(), E>
    where
        E: DeError,
    {
        self.preflight.node(self.depth)
    }

    fn string<E>(&mut self, value: &str) -> Result<(), E>
    where
        E: DeError,
    {
        self.scalar()?;
        self.preflight.string(value.len())
    }

    fn number<E>(&mut self, bytes: usize) -> Result<(), E>
    where
        E: DeError,
    {
        self.scalar()?;
        self.preflight.number(bytes)
    }
}

impl<'de, R> Visitor<'de> for JsonPreflightVisitor<'_, '_, R>
where
    R: Clone,
{
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON value")
    }

    fn visit_bool<E>(mut self, _value: bool) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.scalar()
    }

    fn visit_i64<E>(mut self, value: i64) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.number(value.to_string().len())
    }

    fn visit_u64<E>(mut self, value: u64) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.number(value.to_string().len())
    }

    fn visit_i128<E>(mut self, value: i128) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.number(value.to_string().len())
    }

    fn visit_u128<E>(mut self, value: u128) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.number(value.to_string().len())
    }

    fn visit_f32<E>(mut self, value: f32) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.number(value.to_string().len())
    }

    fn visit_f64<E>(mut self, value: f64) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.number(value.to_string().len())
    }

    fn visit_unit<E>(mut self) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.scalar()
    }

    fn visit_none<E>(mut self) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.scalar()
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        (&mut *self.preflight).deserialize(deserializer)
    }

    fn visit_newtype_struct<D>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        (&mut *self.preflight).deserialize(deserializer)
    }

    fn visit_borrowed_str<E>(
        mut self,
        value: &'de str,
    ) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.string(value)
    }

    fn visit_str<E>(mut self, value: &str) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.string(value)
    }

    fn visit_string<E>(mut self, value: String) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.string(&value)
    }

    fn visit_char<E>(mut self, value: char) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.scalar()?;
        self.preflight.string(value.len_utf8())
    }

    fn visit_seq<A>(mut self, mut access: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        self.scalar()?;
        let mut items = 0_usize;
        while access
            .next_element_seed(JsonPreflightChildSeed {
                preflight: self.preflight,
                depth: self.depth.saturating_add(1),
            })?
            .is_some()
        {
            items = items.saturating_add(1);
            self.preflight.sequence_items(items)?;
        }
        Ok(())
    }

    fn visit_map<A>(mut self, mut access: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        self.scalar()?;
        let Some(first_key) = access.next_key::<String>()? else {
            return Ok(());
        };
        let mut entries = 1_usize;
        self.preflight.map_entries(entries)?;
        self.preflight.key(first_key.len())?;

        if first_key == JSON_NUMBER_TOKEN {
            let number_text = access.next_value::<String>()?;
            let mut next_key = access.next_key::<String>()?;
            if next_key.is_none() {
                return self.preflight.number(number_text.len());
            }
            self.preflight.node(self.depth.saturating_add(1))?;
            self.preflight.string(number_text.len())?;
            while let Some(key) = next_key {
                entries = entries.saturating_add(1);
                self.preflight.map_entries(entries)?;
                self.preflight.key(key.len())?;
                access.next_value_seed(JsonPreflightChildSeed {
                    preflight: self.preflight,
                    depth: self.depth.saturating_add(1),
                })?;
                next_key = access.next_key::<String>()?;
            }
            return Ok(());
        }

        access.next_value_seed(JsonPreflightChildSeed {
            preflight: self.preflight,
            depth: self.depth.saturating_add(1),
        })?;
        while let Some(key) = access.next_key::<String>()? {
            entries = entries.saturating_add(1);
            self.preflight.map_entries(entries)?;
            self.preflight.key(key.len())?;
            access.next_value_seed(JsonPreflightChildSeed {
                preflight: self.preflight,
                depth: self.depth.saturating_add(1),
            })?;
        }
        Ok(())
    }
}

struct JsonPreflightChildSeed<'a, 'b, R> {
    preflight: &'a mut JsonPreflight<'b, R>,
    depth: usize,
}

impl<'de, 'a, 'b, R> DeserializeSeed<'de> for JsonPreflightChildSeed<'a, 'b, R>
where
    R: Clone,
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(JsonPreflightVisitor {
            preflight: self.preflight,
            depth: self.depth,
        })
    }
}
