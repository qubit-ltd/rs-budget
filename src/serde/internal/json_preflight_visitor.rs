// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Serde visitor that charges JSON values during preflight.

use std::fmt;

use serde::de::DeserializeSeed;
use serde::de::Deserializer;
use serde::de::Error as DeError;
use serde::de::MapAccess;
use serde::de::SeqAccess;
use serde::de::Visitor;

use super::JsonPreflight;
use super::JsonPreflightChildSeed;

/// Private token used by `serde_json` for arbitrary-precision numbers.
const JSON_NUMBER_TOKEN: &str =
    concat!("$", "serde_json", ":", ":private::Number");

/// Serde visitor that charges one JSON value against a preflight walker.
///
/// # Type Parameters
///
/// * `R` - Resource identity reported by budget violations.
pub(in crate::serde) struct JsonPreflightVisitor<'a, 'b, R> {
    /// Preflight walker that records budget charges and violations.
    pub(in crate::serde) preflight: &'a mut JsonPreflight<'b, R>,

    /// Inclusive nesting depth of the value being visited.
    pub(in crate::serde) depth: usize,
}

impl<R> JsonPreflightVisitor<'_, '_, R>
where
    R: Clone,
{
    /// Charges one scalar JSON node at the visitor's depth.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the node budget accepts the charge.
    ///
    /// # Errors
    ///
    /// Returns a Serde error after storing the original budget violation.
    ///
    /// # Type Parameters
    ///
    /// * `E` - Serde deserializer error type used to surface the failure.
    fn scalar<E>(&mut self) -> Result<(), E>
    where
        E: DeError,
    {
        self.preflight.node(self.depth)
    }

    /// Charges one JSON string node and its UTF-8 byte length.
    ///
    /// # Parameters
    ///
    /// * `value` - String content whose byte length is charged.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the node and string budgets accept the charges.
    ///
    /// # Errors
    ///
    /// Returns a Serde error after storing the original budget violation.
    ///
    /// # Type Parameters
    ///
    /// * `E` - Serde deserializer error type used to surface the failure.
    fn string<E>(&mut self, value: &str) -> Result<(), E>
    where
        E: DeError,
    {
        self.scalar()?;
        self.preflight.string(value.len())
    }

    /// Charges one JSON number node and its textual byte length.
    ///
    /// # Parameters
    ///
    /// * `bytes` - Byte length of the number's textual representation.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the node and number budgets accept the charges.
    ///
    /// # Errors
    ///
    /// Returns a Serde error after storing the original budget violation.
    ///
    /// # Type Parameters
    ///
    /// * `E` - Serde deserializer error type used to surface the failure.
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

    /// Describes the value expected by this visitor.
    ///
    /// # Parameters
    ///
    /// * `formatter` - Destination for the expectation message.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the message was written.
    ///
    /// # Errors
    ///
    /// Returns a formatting error when the destination rejects the write.
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON value")
    }

    /// Charges one JSON boolean value.
    ///
    /// # Parameters
    ///
    /// * `_value` - Boolean payload; only its presence is charged.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the scalar node budget accepts the charge.
    ///
    /// # Errors
    ///
    /// Returns a Serde error after storing the original budget violation.
    ///
    /// # Type Parameters
    ///
    /// * `E` - Serde deserializer error type used to surface the failure.
    fn visit_bool<E>(mut self, _value: bool) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.scalar()
    }

    /// Charges one signed 64-bit JSON number.
    ///
    /// # Parameters
    ///
    /// * `value` - Number whose decimal representation length is charged.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the node and number budgets accept the charges.
    ///
    /// # Errors
    ///
    /// Returns a Serde error after storing the original budget violation.
    ///
    /// # Type Parameters
    ///
    /// * `E` - Serde deserializer error type used to surface the failure.
    fn visit_i64<E>(mut self, value: i64) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.number(value.to_string().len())
    }

    /// Charges one unsigned 64-bit JSON number.
    ///
    /// # Parameters
    ///
    /// * `value` - Number whose decimal representation length is charged.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the node and number budgets accept the charges.
    ///
    /// # Errors
    ///
    /// Returns a Serde error after storing the original budget violation.
    ///
    /// # Type Parameters
    ///
    /// * `E` - Serde deserializer error type used to surface the failure.
    fn visit_u64<E>(mut self, value: u64) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.number(value.to_string().len())
    }

    /// Charges one signed 128-bit JSON number.
    ///
    /// # Parameters
    ///
    /// * `value` - Number whose decimal representation length is charged.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the node and number budgets accept the charges.
    ///
    /// # Errors
    ///
    /// Returns a Serde error after storing the original budget violation.
    ///
    /// # Type Parameters
    ///
    /// * `E` - Serde deserializer error type used to surface the failure.
    fn visit_i128<E>(mut self, value: i128) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.number(value.to_string().len())
    }

    /// Charges one unsigned 128-bit JSON number.
    ///
    /// # Parameters
    ///
    /// * `value` - Number whose decimal representation length is charged.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the node and number budgets accept the charges.
    ///
    /// # Errors
    ///
    /// Returns a Serde error after storing the original budget violation.
    ///
    /// # Type Parameters
    ///
    /// * `E` - Serde deserializer error type used to surface the failure.
    fn visit_u128<E>(mut self, value: u128) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.number(value.to_string().len())
    }

    /// Charges one 32-bit floating-point JSON number.
    ///
    /// # Parameters
    ///
    /// * `value` - Number whose textual representation length is charged.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the node and number budgets accept the charges.
    ///
    /// # Errors
    ///
    /// Returns a Serde error after storing the original budget violation.
    ///
    /// # Type Parameters
    ///
    /// * `E` - Serde deserializer error type used to surface the failure.
    fn visit_f32<E>(mut self, value: f32) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.number(value.to_string().len())
    }

    /// Charges one 64-bit floating-point JSON number.
    ///
    /// # Parameters
    ///
    /// * `value` - Number whose textual representation length is charged.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the node and number budgets accept the charges.
    ///
    /// # Errors
    ///
    /// Returns a Serde error after storing the original budget violation.
    ///
    /// # Type Parameters
    ///
    /// * `E` - Serde deserializer error type used to surface the failure.
    fn visit_f64<E>(mut self, value: f64) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.number(value.to_string().len())
    }

    /// Charges one JSON null/unit value.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the scalar node budget accepts the charge.
    ///
    /// # Errors
    ///
    /// Returns a Serde error after storing the original budget violation.
    ///
    /// # Type Parameters
    ///
    /// * `E` - Serde deserializer error type used to surface the failure.
    fn visit_unit<E>(mut self) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.scalar()
    }

    /// Charges one absent optional JSON value as a scalar node.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the scalar node budget accepts the charge.
    ///
    /// # Errors
    ///
    /// Returns a Serde error after storing the original budget violation.
    ///
    /// # Type Parameters
    ///
    /// * `E` - Serde deserializer error type used to surface the failure.
    fn visit_none<E>(mut self) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.scalar()
    }

    /// Walks one present optional JSON value through the shared preflight.
    ///
    /// # Parameters
    ///
    /// * `deserializer` - Deserializer positioned at the present value.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the nested value was walked without a Serde failure.
    ///
    /// # Errors
    ///
    /// Returns the deserializer error produced by Serde or by a recorded
    /// budget violation.
    ///
    /// # Type Parameters
    ///
    /// * `D` - Deserializer supplying the present value.
    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        (&mut *self.preflight).deserialize(deserializer)
    }

    /// Walks one newtype JSON value through the shared preflight.
    ///
    /// # Parameters
    ///
    /// * `deserializer` - Deserializer positioned at the newtype payload.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the nested value was walked without a Serde failure.
    ///
    /// # Errors
    ///
    /// Returns the deserializer error produced by Serde or by a recorded
    /// budget violation.
    ///
    /// # Type Parameters
    ///
    /// * `D` - Deserializer supplying the newtype payload.
    fn visit_newtype_struct<D>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        (&mut *self.preflight).deserialize(deserializer)
    }

    /// Charges one borrowed JSON string.
    ///
    /// # Parameters
    ///
    /// * `value` - Borrowed string content whose byte length is charged.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the node and string budgets accept the charges.
    ///
    /// # Errors
    ///
    /// Returns a Serde error after storing the original budget violation.
    ///
    /// # Type Parameters
    ///
    /// * `E` - Serde deserializer error type used to surface the failure.
    fn visit_borrowed_str<E>(
        mut self,
        value: &'de str,
    ) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.string(value)
    }

    /// Charges one JSON string view.
    ///
    /// # Parameters
    ///
    /// * `value` - String content whose byte length is charged.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the node and string budgets accept the charges.
    ///
    /// # Errors
    ///
    /// Returns a Serde error after storing the original budget violation.
    ///
    /// # Type Parameters
    ///
    /// * `E` - Serde deserializer error type used to surface the failure.
    fn visit_str<E>(mut self, value: &str) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.string(value)
    }

    /// Charges one owned JSON string.
    ///
    /// # Parameters
    ///
    /// * `value` - Owned string content whose byte length is charged.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the node and string budgets accept the charges.
    ///
    /// # Errors
    ///
    /// Returns a Serde error after storing the original budget violation.
    ///
    /// # Type Parameters
    ///
    /// * `E` - Serde deserializer error type used to surface the failure.
    fn visit_string<E>(mut self, value: String) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.string(&value)
    }

    /// Charges one JSON character as a scalar string of its UTF-8 length.
    ///
    /// # Parameters
    ///
    /// * `value` - Character whose UTF-8 byte length is charged.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the node and string budgets accept the charges.
    ///
    /// # Errors
    ///
    /// Returns a Serde error after storing the original budget violation.
    ///
    /// # Type Parameters
    ///
    /// * `E` - Serde deserializer error type used to surface the failure.
    fn visit_char<E>(mut self, value: char) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.scalar()?;
        self.preflight.string(value.len_utf8())
    }

    /// Charges one JSON sequence and each nested element.
    ///
    /// # Parameters
    ///
    /// * `access` - Sequence access used to walk nested elements.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the sequence and all elements were charged successfully.
    ///
    /// # Errors
    ///
    /// Returns a Serde error after a nested walk failure or budget violation.
    ///
    /// # Type Parameters
    ///
    /// * `A` - Sequence access implementation supplied by Serde.
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

    /// Charges one JSON map, including arbitrary-precision number maps.
    ///
    /// # Parameters
    ///
    /// * `access` - Map access used to walk keys and nested values.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the map and all entries were charged successfully.
    ///
    /// # Errors
    ///
    /// Returns a Serde error after a nested walk failure or budget violation.
    ///
    /// # Type Parameters
    ///
    /// * `A` - Map access implementation supplied by Serde.
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
