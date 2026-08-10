// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
//    You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
//    Unless required by applicable law or agreed to in writing, software
//    distributed under the License is distributed on an "AS IS" BASIS,
//    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//    See the License for the specific language governing permissions and
//    limitations under the License.
// =============================================================================
//! Single-pass string rendering with transactional budget accounting.

use std::fmt;
use std::io;

use super::BudgetedStringError;
use crate::BudgetError;
use crate::ResourceBudget;

enum WriterFailure<R> {
    Budget(BudgetError<R, usize>),
    LengthOverflow,
}

/// Collects a UTF-8 string while checking a finite byte budget incrementally.
///
/// The writer is constructed and committed by
/// [`ResourceBudget::try_write_string`]. A failed render drops the buffered
/// prefix and leaves the budget unchanged.
pub struct BudgetedStringWriter<'a, R> {
    budget: &'a ResourceBudget<R, usize>,
    output: Vec<u8>,
    failure: Option<WriterFailure<R>>,
}

impl<'a, R> BudgetedStringWriter<'a, R>
where
    R: Clone,
{
    fn new(budget: &'a ResourceBudget<R, usize>) -> Self {
        Self {
            budget,
            output: Vec::new(),
            failure: None,
        }
    }

    fn into_parts(self) -> (Vec<u8>, Option<WriterFailure<R>>) {
        (self.output, self.failure)
    }

    fn append(&mut self, bytes: &[u8]) -> bool {
        if self.failure.is_some() {
            return false;
        }
        let Some(next_len) = checked_output_len(self.output.len(), bytes.len())
        else {
            self.failure = Some(WriterFailure::LengthOverflow);
            return false;
        };
        if let Err(error) = self.budget.check_available(next_len) {
            self.failure = Some(WriterFailure::Budget(error));
            return false;
        }
        if next_len > self.output.capacity() {
            let target = self
                .output
                .capacity()
                .saturating_mul(2)
                .max(next_len)
                .min(self.budget.remaining());
            self.output
                .reserve_exact(target.saturating_sub(self.output.len()));
        }
        self.output.extend_from_slice(bytes);
        true
    }

    /// Returns a formatting writer view over the current transaction.
    #[inline]
    pub fn as_fmt(&mut self) -> impl fmt::Write + '_ {
        FmtWriter { writer: self }
    }

    /// Returns an I/O writer view over the current transaction.
    #[inline]
    pub fn as_io(&mut self) -> impl io::Write + '_ {
        IoWriter { writer: self }
    }
}

struct FmtWriter<'writer, 'budget, R> {
    writer: &'writer mut BudgetedStringWriter<'budget, R>,
}

impl<R> fmt::Write for FmtWriter<'_, '_, R>
where
    R: Clone,
{
    fn write_str(&mut self, value: &str) -> fmt::Result {
        if self.writer.append(value.as_bytes()) {
            Ok(())
        } else {
            Err(fmt::Error)
        }
    }
}

struct IoWriter<'writer, 'budget, R> {
    writer: &'writer mut BudgetedStringWriter<'budget, R>,
}

impl<R> io::Write for IoWriter<'_, '_, R>
where
    R: Clone,
{
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.writer.append(bytes) {
            Ok(bytes.len())
        } else {
            Err(io::Error::other("budgeted string writer rejected output"))
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

const fn checked_output_len(
    current: usize,
    additional: usize,
) -> Option<usize> {
    current.checked_add(additional)
}

impl<R> ResourceBudget<R, usize>
where
    R: Clone + fmt::Debug,
{
    /// Renders and transactionally commits a UTF-8 string under this budget.
    pub fn try_write_string<E, F>(
        &mut self,
        render: F,
    ) -> Result<String, BudgetedStringError<R, E>>
    where
        E: fmt::Debug + fmt::Display,
        F: FnOnce(&mut BudgetedStringWriter<'_, R>) -> Result<(), E>,
    {
        let mut writer = BudgetedStringWriter::new(self);
        let rendered = render(&mut writer);
        let (bytes, failure) = writer.into_parts();
        match failure {
            Some(WriterFailure::Budget(error)) => {
                return Err(BudgetedStringError::Budget(error));
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
        self.try_consume(output.len())
            .map_err(BudgetedStringError::Budget)?;
        Ok(output)
    }
}
