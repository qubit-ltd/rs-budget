// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Describes whether a reported resource measurement is exact or conservative.
// qubit-style: allow source-test-pair

use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

/// A resource observation that may be exact or only a safe lower bound.
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
    #[must_use]
    pub const fn exact(self) -> Option<Q> {
        match self {
            Self::Exact(value) => Some(value),
            Self::AtLeast(_) => None,
        }
    }

    /// Returns the safe lower bound represented by this observation.
    #[inline(always)]
    #[must_use]
    pub const fn lower_bound(self) -> Q {
        match self {
            Self::Exact(value) | Self::AtLeast(value) => value,
        }
    }
}
