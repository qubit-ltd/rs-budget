// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Compound wrappers used by the budget-aware JSON serializer.

use std::cell::RefCell;
use std::rc::Rc;

use serde::Serialize;
use serde::Serializer;
use serde::ser::SerializeMap;
use serde::ser::SerializeSeq;
use serde::ser::SerializeStruct;
use serde::ser::SerializeStructVariant;
use serde::ser::SerializeTuple;
use serde::ser::SerializeTupleStruct;
use serde::ser::SerializeTupleVariant;

use super::JsonBudgetSerializer;
use super::json_budget_serializer::JsonBudgetContext;
use crate::BudgetError;

/// Special serde_json struct encoding recognized by the wrapper.
#[derive(Clone, Copy)]
enum PrivateStruct {
    /// Regular JSON object encoding.
    Regular,

    /// Arbitrary-precision JSON number encoding.
    Number,

    /// Raw JSON fragment encoding.
    RawValue,
}

/// Wraps one nested value so the underlying compound serializer re-enters the
/// budget-aware serializer before traversing it.
pub(super) struct BudgetedValue<'a, 'budget, T, R>
where
    T: ?Sized,
{
    /// Original nested value.
    value: &'a T,

    /// Shared mutable budget state for the serialization traversal.
    context: Rc<RefCell<JsonBudgetContext<'budget, R>>>,

    /// Root-inclusive depth assigned to the nested value.
    depth: usize,
}

impl<'a, 'budget, T, R> BudgetedValue<'a, 'budget, T, R>
where
    T: ?Sized,
{
    /// Creates a nested value wrapper bound to a shared budget context.
    pub(super) const fn new(
        value: &'a T,
        context: Rc<RefCell<JsonBudgetContext<'budget, R>>>,
        depth: usize,
    ) -> Self {
        Self {
            value,
            context,
            depth,
        }
    }
}

impl<T, R> Serialize for BudgetedValue<'_, '_, T, R>
where
    T: Serialize + ?Sized,
    R: Clone,
{
    /// Serializes the wrapped value through a child budget decorator.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.value.serialize(JsonBudgetSerializer::with_context(
            serializer,
            Rc::clone(&self.context),
            self.depth,
        ))
    }
}

/// Wraps a Serde compound serializer and checks container operations before
/// delegating them.
pub(in crate::serde) struct JsonBudgetCompound<'a, C, R> {
    /// Underlying Serde compound serializer.
    inner: C,

    /// Shared mutable budget state for the serialization traversal.
    context: Rc<RefCell<JsonBudgetContext<'a, R>>>,

    /// Root-inclusive depth assigned to nested values.
    child_depth: usize,

    /// Number of sequence items or map entries observed so far.
    observed: usize,

    /// Private serde_json encoding represented by this compound.
    private: PrivateStruct,
}

impl<'a, C, R> JsonBudgetCompound<'a, C, R>
where
    R: Clone,
{
    /// Creates a wrapper for a regular JSON array or object compound.
    pub(super) const fn new(
        inner: C,
        context: Rc<RefCell<JsonBudgetContext<'a, R>>>,
        child_depth: usize,
    ) -> Self {
        Self {
            inner,
            context,
            child_depth,
            observed: 0,
            private: PrivateStruct::Regular,
        }
    }

    /// Creates a wrapper for serde_json's private number compound.
    pub(super) const fn number(
        inner: C,
        context: Rc<RefCell<JsonBudgetContext<'a, R>>>,
        depth: usize,
    ) -> Self {
        Self {
            inner,
            context,
            child_depth: depth,
            observed: 0,
            private: PrivateStruct::Number,
        }
    }

    /// Creates a wrapper for serde_json's private raw-value compound.
    pub(super) const fn raw_value(
        inner: C,
        context: Rc<RefCell<JsonBudgetContext<'a, R>>>,
        depth: usize,
    ) -> Self {
        Self {
            inner,
            context,
            child_depth: depth,
            observed: 0,
            private: PrivateStruct::RawValue,
        }
    }

    /// Records the original budget error and maps it into the compound error.
    fn record<E>(&mut self, result: Result<(), BudgetError<R, usize>>) -> Result<(), E>
    where
        E: serde::ser::Error,
    {
        self.context.borrow_mut().record(result)
    }

    /// Checks the next observed sequence element.
    fn next_sequence<E>(&mut self) -> Result<(), E>
    where
        E: serde::ser::Error,
    {
        self.observed = self.observed.saturating_add(1);
        let result = self
            .context
            .borrow()
            .budget
            .check_sequence_items(self.observed);
        self.record(result)
    }

    /// Checks the next observed map or struct entry.
    fn next_map_entry<E>(&mut self) -> Result<(), E>
    where
        E: serde::ser::Error,
    {
        self.observed = self.observed.saturating_add(1);
        let result = self
            .context
            .borrow()
            .budget
            .check_map_entries(self.observed);
        self.record(result)
    }
}

impl<C, R> SerializeSeq for JsonBudgetCompound<'_, C, R>
where
    C: SerializeSeq,
    R: Clone,
{
    type Ok = C::Ok;
    type Error = C::Error;

    /// Checks the observed count, then serializes one decorated child value.
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.next_sequence()?;
        let value = BudgetedValue::new(value, Rc::clone(&self.context), self.child_depth);
        self.inner.serialize_element(&value)
    }

    /// Completes the underlying sequence.
    #[inline(always)]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.inner.end()
    }
}

impl<C, R> SerializeTuple for JsonBudgetCompound<'_, C, R>
where
    C: SerializeTuple,
    R: Clone,
{
    type Ok = C::Ok;
    type Error = C::Error;

    /// Serializes one decorated tuple element.
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.next_sequence()?;
        let value = BudgetedValue::new(value, Rc::clone(&self.context), self.child_depth);
        self.inner.serialize_element(&value)
    }

    /// Completes the underlying tuple.
    #[inline(always)]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.inner.end()
    }
}

impl<C, R> SerializeTupleStruct for JsonBudgetCompound<'_, C, R>
where
    C: SerializeTupleStruct,
    R: Clone,
{
    type Ok = C::Ok;
    type Error = C::Error;

    /// Serializes one decorated tuple-struct field.
    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.next_sequence()?;
        let value = BudgetedValue::new(value, Rc::clone(&self.context), self.child_depth);
        self.inner.serialize_field(&value)
    }

    /// Completes the underlying tuple struct.
    #[inline(always)]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.inner.end()
    }
}

impl<C, R> SerializeTupleVariant for JsonBudgetCompound<'_, C, R>
where
    C: SerializeTupleVariant,
    R: Clone,
{
    type Ok = C::Ok;
    type Error = C::Error;

    /// Serializes one decorated tuple-variant field.
    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.next_sequence()?;
        let value = BudgetedValue::new(value, Rc::clone(&self.context), self.child_depth);
        self.inner.serialize_field(&value)
    }

    /// Completes the underlying tuple variant.
    #[inline(always)]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.inner.end()
    }
}

impl<C, R> SerializeMap for JsonBudgetCompound<'_, C, R>
where
    C: SerializeMap,
    R: Clone,
{
    type Ok = C::Ok;
    type Error = C::Error;

    /// Checks the entry count and key before delegating key serialization.
    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.next_map_entry()?;
        let key = super::json_budget_serializer::BudgetedKey::new(key, Rc::clone(&self.context));
        self.inner.serialize_key(&key)
    }

    /// Serializes one map value through a child budget decorator.
    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        let value = BudgetedValue::new(value, Rc::clone(&self.context), self.child_depth);
        self.inner.serialize_value(&value)
    }

    /// Checks and serializes one complete map entry.
    fn serialize_entry<K, V>(&mut self, key: &K, value: &V) -> Result<(), Self::Error>
    where
        K: Serialize + ?Sized,
        V: Serialize + ?Sized,
    {
        self.serialize_key(key)?;
        self.serialize_value(value)
    }

    /// Completes the underlying map.
    #[inline(always)]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.inner.end()
    }
}

impl<C, R> SerializeStruct for JsonBudgetCompound<'_, C, R>
where
    C: SerializeStruct,
    R: Clone,
{
    type Ok = C::Ok;
    type Error = C::Error;

    /// Checks one field key and serializes its decorated value.
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        match self.private {
            PrivateStruct::Number => {
                let value = super::json_budget_serializer::BudgetedPrivateValue::number(
                    value,
                    Rc::clone(&self.context),
                );
                return self.inner.serialize_field(key, &value);
            }
            PrivateStruct::RawValue => {
                let value = super::json_budget_serializer::BudgetedPrivateValue::raw_value(
                    value,
                    Rc::clone(&self.context),
                    self.child_depth,
                );
                return self.inner.serialize_field(key, &value);
            }
            PrivateStruct::Regular => self.next_map_entry()?,
        }
        let key_result = self.context.borrow().budget.check_key_bytes(key.len());
        self.record(key_result)?;
        let value = BudgetedValue::new(value, Rc::clone(&self.context), self.child_depth);
        self.inner.serialize_field(key, &value)
    }

    /// Skips one field exactly as the underlying serializer requests.
    #[inline(always)]
    fn skip_field(&mut self, key: &'static str) -> Result<(), Self::Error> {
        self.inner.skip_field(key)
    }

    /// Completes the underlying struct.
    #[inline(always)]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.inner.end()
    }
}

impl<C, R> SerializeStructVariant for JsonBudgetCompound<'_, C, R>
where
    C: SerializeStructVariant,
    R: Clone,
{
    type Ok = C::Ok;
    type Error = C::Error;

    /// Checks one field key and serializes its decorated value.
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.next_map_entry()?;
        let key_result = self.context.borrow().budget.check_key_bytes(key.len());
        self.record(key_result)?;
        let value = BudgetedValue::new(value, Rc::clone(&self.context), self.child_depth);
        self.inner.serialize_field(key, &value)
    }

    /// Skips one variant field exactly as the underlying serializer requests.
    #[inline(always)]
    fn skip_field(&mut self, key: &'static str) -> Result<(), Self::Error> {
        self.inner.skip_field(key)
    }

    /// Completes the underlying struct variant.
    #[inline(always)]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.inner.end()
    }
}
