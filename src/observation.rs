// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
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
//! Describes whether a reported resource measurement is exact or conservative.
// qubit-style: allow source-test-pair

use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

/// A resource observation that may be exact or only a safe lower bound.
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Observation<Q> {
    /// The measured quantity is exact.
    Exact(Q),

    /// The measured quantity is at least the contained lower bound.
    AtLeast(Q),
}

impl<Q> Display for Observation<Q>
where
    Q: Display,
{
    /// Formats the observation with its precision qualifier.
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Exact(value) => write!(formatter, "exactly {value}"),
            Self::AtLeast(value) => write!(formatter, "at least {value}"),
        }
    }
}

impl<Q> Observation<Q>
where
    Q: Copy + Debug,
{
    /// Returns the exact quantity, or `None` for a lower-bound observation.
    #[inline(always)]
    pub const fn exact(self) -> Option<Q> {
        match self {
            Self::Exact(value) => Some(value),
            Self::AtLeast(_) => None,
        }
    }

    /// Returns the safe lower bound represented by this observation.
    #[inline(always)]
    pub const fn lower_bound(self) -> Q {
        match self {
            Self::Exact(value) | Self::AtLeast(value) => value,
        }
    }
}
