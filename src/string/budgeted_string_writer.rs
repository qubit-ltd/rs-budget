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
use crate::resource::ResourceBudget;
use crate::resource::ResourceQuantity;

/// Collects a UTF-8 string while checking a finite byte budget incrementally.
///
/// The writer is constructed and committed by
/// [`ResourceBudget::try_write_string`]. A failed render drops the buffered
/// prefix and leaves the budget unchanged.
///
/// Use [`Self::as_fmt`] for formatting APIs or [`Self::as_io`] for
/// byte-oriented I/O. The returned adapter borrows this writer, so the render
/// callback must finish using it before returning.
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource identity retained by limits and errors.
/// * `Q` - Exact unsigned quantity used for measurements and accounting.
///
/// # Examples
///
/// ```
/// use std::fmt::Write as _;
/// use qubit_budget::ResourceBudget;
///
/// let mut budget = ResourceBudget::new("response bytes", 16_u64);
/// let output = budget
///     .try_write_string(|writer| {
///         let mut formatted = writer.as_fmt();
///         write!(&mut formatted, "status={}", 200)
///     })
///     .expect("the rendered response should fit");
///
/// assert_eq!(output, "status=200");
/// assert_eq!(budget.used(), 10);
/// ```
pub struct BudgetedStringWriter<'a, R, Q = u64>
where
    Q: ResourceQuantity,
{
    /// Immutable budget snapshot used to validate the final output length.
    budget: &'a ResourceBudget<R, Q>,
    /// Bytes staged until the entire rendering transaction succeeds.
    output: Vec<u8>,
    /// First writer-side failure retained for deterministic error precedence.
    failure: Option<WriterFailure<R, Q>>,
}

impl<'a, R, Q> BudgetedStringWriter<'a, R, Q>
where
    R: Clone,
    Q: ResourceQuantity,
{
    /// Creates an empty writer backed by an immutable budget snapshot.
    ///
    /// # Parameters
    ///
    /// * `budget` - Immutable capacity snapshot used to validate output.
    ///
    /// # Returns
    ///
    /// Creates an empty writer backed by an immutable budget snapshot.
    fn new(budget: &'a ResourceBudget<R, Q>) -> Self {
        Self {
            budget,
            output: Vec::new(),
            failure: None,
        }
    }

    /// Separates rendered bytes from the first captured failure.
    ///
    /// # Returns
    ///
    /// Separates rendered bytes from the first captured failure.
    ///
    /// A `None` failure component indicates that no writer-side failure was
    /// captured.
    fn into_parts(self) -> (Vec<u8>, Option<WriterFailure<R, Q>>) {
        (self.output, self.failure)
    }

    /// Appends bytes after checking the next cumulative length and budget.
    ///
    /// Returns `true` when the bytes were appended, or `false` after storing
    /// the first failure for the enclosing transaction.
    ///
    /// # Parameters
    ///
    /// * `bytes` - Bytes to append to the transactional output.
    ///
    /// # Returns
    ///
    /// Appends bytes after checking the next cumulative length and budget.
    pub(crate) fn append(&mut self, bytes: &[u8]) -> bool {
        if self.failure.is_some() {
            return false;
        }
        let Some(next_len) = checked_output_len(self.output.len(), bytes.len()) else {
            self.failure = Some(WriterFailure::LengthOverflow);
            return false;
        };
        let next_length = match Q::try_from_usize(next_len) {
            Ok(value) => value,
            Err(source) => {
                self.failure = Some(WriterFailure::Quantity {
                    resource: self.budget.resource().clone(),
                    source,
                });
                return false;
            }
        };
        if let Err(error) = self.budget.check_available(next_length) {
            self.failure = Some(WriterFailure::Budget(error));
            return false;
        }
        if next_len > self.output.capacity() {
            let target = self.output.capacity().saturating_mul(2).max(next_len);
            if let Err(source) = self.output.try_reserve_exact(target.saturating_sub(self.output.len())) {
                self.failure = Some(WriterFailure::Allocation(source));
                return false;
            }
        }
        self.output.extend_from_slice(bytes);
        true
    }

    /// Returns a formatting writer view over the current transaction.
    ///
    /// # Returns
    ///
    /// Returns a formatting writer view over the current transaction.
    #[must_use]
    #[inline]
    pub fn as_fmt(&mut self) -> impl fmt::Write + '_ {
        FmtWriter { writer: self }
    }

    /// Returns an I/O writer view over the current transaction.
    ///
    /// # Returns
    ///
    /// Returns an I/O writer view over the current transaction.
    #[must_use]
    #[inline]
    pub fn as_io(&mut self) -> impl io::Write + '_ {
        IoWriter { writer: self }
    }
}

/// Adds two output lengths while detecting `usize` overflow.
///
/// # Parameters
///
/// * `current` - Bytes already staged in the output buffer.
/// * `additional` - Bytes requested by the next append.
///
/// # Returns
///
/// Adds two output lengths while detecting `usize` overflow.
///
/// `None` indicates that the arithmetic sum would overflow `usize`.
const fn checked_output_len(current: usize, additional: usize) -> Option<usize> {
    current.checked_add(additional)
}

/// Renders a UTF-8 string and commits its byte length to a resource budget.
///
/// This crate-internal implementation keeps string buffering and writer error
/// precedence in the string domain. [`ResourceBudget::try_write_string`]
/// exposes the public type-owned forwarding method.
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource identity retained by limits and errors.
/// * `Q` - Exact unsigned quantity used for byte accounting.
/// * `E` - Error type returned by the caller-provided renderer.
/// * `F` - Closure that renders into the transactional writer.
///
/// # Parameters
///
/// * `budget` - Finite byte budget charged only after rendering succeeds.
/// * `render` - Caller-provided renderer writing into the transactional
///   adapter.
///
/// # Returns
///
/// `Ok(rendered)` after the complete UTF-8 output is charged and committed.
///
/// # Errors
///
/// Returns [`BudgetedStringError`] when rendering, allocation, UTF-8
/// validation, measurement, or budget accounting fails.
pub(crate) fn render_budgeted_string<R, Q, E, F>(
    budget: &mut ResourceBudget<R, Q>,
    render: F,
) -> Result<String, BudgetedStringError<R, E, Q>>
where
    R: Clone + fmt::Debug,
    Q: ResourceQuantity,
    E: fmt::Debug + fmt::Display,
    F: FnOnce(&mut BudgetedStringWriter<'_, R, Q>) -> Result<(), E>,
{
    let mut writer = BudgetedStringWriter::new(budget);
    let rendered = render(&mut writer);
    let (bytes, failure) = writer.into_parts();
    match failure {
        Some(WriterFailure::Budget(error)) => {
            return Err(BudgetedStringError::Budget(error));
        }
        Some(WriterFailure::Quantity { resource, source }) => {
            return Err(BudgetedStringError::Quantity { resource, source });
        }
        Some(WriterFailure::LengthOverflow) => {
            return Err(BudgetedStringError::LengthOverflow);
        }
        Some(WriterFailure::Allocation(source)) => {
            return Err(BudgetedStringError::Allocation(source));
        }
        None => {}
    }
    if let Err(error) = rendered {
        return Err(BudgetedStringError::Render(error));
    }
    let output = String::from_utf8(bytes).map_err(BudgetedStringError::InvalidUtf8)?;
    let output_length = Q::try_from_usize(output.len()).map_err(|source| BudgetedStringError::Quantity {
        resource: budget.resource().clone(),
        source,
    })?;
    budget.try_consume(output_length).map_err(BudgetedStringError::Budget)?;
    Ok(output)
}
