// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Single-pass string rendering with transactional budget accounting.

use std::fmt;
use std::io;

use super::BudgetedStringError;
use super::internal::FmtWriter;
use super::internal::IoWriter;
use super::internal::WriterFailure;
use crate::MeasuredBudgetError;
use crate::ResourceBudget;
use crate::ResourceQuantity;

/// Collects a UTF-8 string while checking a finite byte budget incrementally.
///
/// The writer is constructed and committed by
/// [`ResourceBudget::try_write_string`]. A failed render drops the buffered
/// prefix and leaves the budget unchanged.
pub struct BudgetedStringWriter<'a, R, Q = u64>
where
    Q: ResourceQuantity,
{
    budget: &'a ResourceBudget<R, Q>,
    output: Vec<u8>,
    failure: Option<WriterFailure<R, Q>>,
}

impl<'a, R, Q> BudgetedStringWriter<'a, R, Q>
where
    R: Clone,
    Q: ResourceQuantity,
{
    /// Creates an empty writer backed by an immutable budget snapshot.
    fn new(budget: &'a ResourceBudget<R, Q>) -> Self {
        Self {
            budget,
            output: Vec::new(),
            failure: None,
        }
    }

    /// Separates rendered bytes from the first captured failure.
    fn into_parts(self) -> (Vec<u8>, Option<WriterFailure<R, Q>>) {
        (self.output, self.failure)
    }

    /// Appends bytes after checking the next cumulative length and budget.
    ///
    /// Returns `true` when the bytes were appended, or `false` after storing
    /// the first failure for the enclosing transaction.
    pub(crate) fn append(&mut self, bytes: &[u8]) -> bool {
        if self.failure.is_some() {
            return false;
        }
        let Some(next_len) = checked_output_len(self.output.len(), bytes.len())
        else {
            self.failure = Some(WriterFailure::LengthOverflow);
            return false;
        };
        let next_length = match Q::try_from_usize(next_len) {
            Ok(value) => value,
            Err(source) => {
                self.failure =
                    Some(WriterFailure::Budget(MeasuredBudgetError::quantity(
                        self.budget.resource().clone(),
                        source,
                    )));
                return false;
            }
        };
        if let Err(error) = self.budget.check_available(next_length) {
            self.failure = Some(WriterFailure::Budget(error.into()));
            return false;
        }
        if next_len > self.output.capacity() {
            let target = self.output.capacity().saturating_mul(2).max(next_len);
            self.output
                .reserve_exact(target.saturating_sub(self.output.len()));
        }
        self.output.extend_from_slice(bytes);
        true
    }

    /// Returns a formatting writer view over the current transaction.
    #[must_use = "formatted output is written through the returned adapter"]
    #[inline]
    pub fn as_fmt(&mut self) -> impl fmt::Write + '_ {
        FmtWriter { writer: self }
    }

    /// Returns an I/O writer view over the current transaction.
    #[must_use = "output bytes are written through the returned adapter"]
    #[inline]
    pub fn as_io(&mut self) -> impl io::Write + '_ {
        IoWriter { writer: self }
    }
}

/// Adds two output lengths while detecting `usize` overflow.
const fn checked_output_len(
    current: usize,
    additional: usize,
) -> Option<usize> {
    current.checked_add(additional)
}

impl<R, Q> ResourceBudget<R, Q>
where
    R: Clone + fmt::Debug,
    Q: ResourceQuantity,
{
    /// Renders and transactionally commits a UTF-8 string under this budget.
    pub fn try_write_string<E, F>(
        &mut self,
        render: F,
    ) -> Result<String, BudgetedStringError<R, E, Q>>
    where
        E: fmt::Debug + fmt::Display,
        F: FnOnce(&mut BudgetedStringWriter<'_, R, Q>) -> Result<(), E>,
    {
        let mut writer = BudgetedStringWriter::new(self);
        let rendered = render(&mut writer);
        let (bytes, failure) = writer.into_parts();
        match failure {
            Some(WriterFailure::Budget(MeasuredBudgetError::Budget(error))) => {
                return Err(BudgetedStringError::Budget(error));
            }
            Some(WriterFailure::Budget(MeasuredBudgetError::Quantity {
                resource,
                source,
            })) => {
                return Err(BudgetedStringError::Quantity { resource, source });
            }
            Some(WriterFailure::LengthOverflow) => {
                return Err(BudgetedStringError::LengthOverflow);
            }
            None => {}
        }
        if let Err(error) = rendered {
            return Err(BudgetedStringError::Render(error));
        }
        let output = String::from_utf8(bytes)
            .map_err(BudgetedStringError::InvalidUtf8)?;
        let output_length =
            Q::try_from_usize(output.len()).map_err(|source| {
                BudgetedStringError::Quantity {
                    resource: self.resource().clone(),
                    source,
                }
            })?;
        self.try_consume(output_length)
            .map_err(BudgetedStringError::Budget)?;
        Ok(output)
    }
}
