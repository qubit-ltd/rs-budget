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
//! Errors returned by transactional string rendering.

use std::fmt::Debug;
use std::fmt::Display;
use std::string::FromUtf8Error;

use thiserror::Error;

use crate::BudgetError;

/// Describes why a budgeted string rendering transaction failed.
#[derive(Debug, Error)]
pub enum BudgetedStringError<R, E>
where
    R: Debug,
    E: Debug + Display,
{
    /// The rendered prefix exceeded the remaining resource budget.
    #[error(transparent)]
    Budget(BudgetError<R, usize>),
    /// The renderer returned an error unrelated to the budget writer.
    #[error("string renderer failed: {0}")]
    Render(E),
    /// The renderer produced bytes that are not valid UTF-8.
    #[error("rendered bytes are not valid UTF-8")]
    InvalidUtf8(#[source] FromUtf8Error),
    /// The rendered byte length overflowed `usize`.
    #[error("rendered string length overflowed usize")]
    LengthOverflow,
}
