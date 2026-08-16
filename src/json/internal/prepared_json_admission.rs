// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Prepares native JSON measurements for deterministic budget admission.

use super::super::JsonMeasurement;
use super::super::JsonValueLimits;
use crate::MeasuredBudgetError;
use crate::ResourceLimit;
use crate::ResourceQuantity;

/// Native JSON measurement converted for the dimensions configured by limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::json) enum PreparedJsonAdmission<Q>
where
    Q: ResourceQuantity,
{
    /// A null value with its converted depth.
    Null { depth: Q },
    /// A boolean value with its converted depth.
    Boolean { depth: Q },
    /// A string value with its converted depth and byte length.
    String { depth: Q, bytes: Q },
    /// A number value with its converted depth and byte length.
    Number { depth: Q, bytes: Q },
    /// An array with its converted depth and item count.
    Array { depth: Q, items: Q },
    /// An object with its converted depth and entry count.
    Object { depth: Q, entries: Q },
    /// An object key with its converted byte length.
    Key { bytes: Q },
}

impl<Q> PreparedJsonAdmission<Q>
where
    Q: ResourceQuantity,
{
    /// Converts a measurement only for dimensions configured by `limits`.
    ///
    /// Returns a conversion error associated with the first configured
    /// resource whose native measurement does not fit `Q`.
    pub(in crate::json) fn prepare<R>(
        limits: &JsonValueLimits<R, Q>,
        measurement: JsonMeasurement,
    ) -> Result<Self, MeasuredBudgetError<R, Q>>
    where
        R: Clone,
    {
        let structure = limits.structure_limits();
        match measurement {
            JsonMeasurement::Null { depth } => Ok(Self::Null {
                depth: convert(depth, structure.depth_limit())?,
            }),
            JsonMeasurement::Boolean { depth } => Ok(Self::Boolean {
                depth: convert(depth, structure.depth_limit())?,
            }),
            JsonMeasurement::String { depth, bytes } => Ok(Self::String {
                depth: convert(depth, structure.depth_limit())?,
                bytes: convert_payload(
                    bytes,
                    limits.string_bytes_limit(),
                    limits.payload_bytes_limit(),
                )?,
            }),
            JsonMeasurement::Number { depth, bytes } => Ok(Self::Number {
                depth: convert(depth, structure.depth_limit())?,
                bytes: convert_payload(
                    bytes,
                    limits.number_bytes_limit(),
                    limits.payload_bytes_limit(),
                )?,
            }),
            JsonMeasurement::Array { depth, items } => Ok(Self::Array {
                depth: convert(depth, structure.depth_limit())?,
                items: convert(items, structure.sequence_items_limit())?,
            }),
            JsonMeasurement::Object { depth, entries } => Ok(Self::Object {
                depth: convert(depth, structure.depth_limit())?,
                entries: convert(entries, structure.map_entries_limit())?,
            }),
            JsonMeasurement::Key { bytes } => Ok(Self::Key {
                bytes: convert_payload(
                    bytes,
                    structure.key_bytes_limit(),
                    limits.payload_bytes_limit(),
                )?,
            }),
        }
    }

    /// Checks configured point limits after native measurement conversion.
    ///
    /// Returns the first depth or variant-specific limit error. This method
    /// neither creates nor mutates a budget.
    pub(in crate::json) fn check_point<R>(
        &self,
        limits: &JsonValueLimits<R, Q>,
    ) -> Result<(), MeasuredBudgetError<R, Q>>
    where
        R: Clone,
    {
        let structure = limits.structure_limits();
        match self {
            Self::Null { depth } | Self::Boolean { depth } => {
                check_limit(structure.depth_limit(), *depth)
            }
            Self::String { depth, bytes } => {
                check_limit(structure.depth_limit(), *depth)?;
                check_limit(limits.string_bytes_limit(), *bytes)
            }
            Self::Number { depth, bytes } => {
                check_limit(structure.depth_limit(), *depth)?;
                check_limit(limits.number_bytes_limit(), *bytes)
            }
            Self::Array { depth, items } => {
                check_limit(structure.depth_limit(), *depth)?;
                check_limit(structure.sequence_items_limit(), *items)
            }
            Self::Object { depth, entries } => {
                check_limit(structure.depth_limit(), *depth)?;
                check_limit(structure.map_entries_limit(), *entries)
            }
            Self::Key { bytes } => check_limit(structure.key_bytes_limit(), *bytes),
        }
    }
}

/// Converts a native quantity when its directly associated limit is set.
///
/// Returns a conversion error carrying the configured limit's resource when
/// `amount` cannot be represented by `Q`.
fn convert<R, Q>(
    amount: usize,
    limit: Option<&ResourceLimit<R, Q>>,
) -> Result<Q, MeasuredBudgetError<R, Q>>
where
    R: Clone,
    Q: ResourceQuantity,
{
    let Some(limit) = limit else {
        return Ok(Q::ZERO);
    };
    Q::try_from_usize(amount)
        .map_err(|source| MeasuredBudgetError::quantity(limit.resource().clone(), source))
}

/// Converts a native payload quantity when either point or cumulative limits
/// are set.
///
/// Returns a conversion error carrying the point-limit resource when present,
/// otherwise the cumulative payload-limit resource.
fn convert_payload<R, Q>(
    amount: usize,
    point_limit: Option<&ResourceLimit<R, Q>>,
    payload_limit: Option<&ResourceLimit<R, Q>>,
) -> Result<Q, MeasuredBudgetError<R, Q>>
where
    R: Clone,
    Q: ResourceQuantity,
{
    if let Some(limit) = point_limit {
        return Q::try_from_usize(amount)
            .map_err(|source| MeasuredBudgetError::quantity(limit.resource().clone(), source));
    }
    convert(amount, payload_limit)
}

/// Checks an optional point limit without changing accounting state.
///
/// Returns the configured resource-limit error when `actual` exceeds its
/// inclusive maximum.
fn check_limit<R, Q>(
    limit: Option<&ResourceLimit<R, Q>>,
    actual: Q,
) -> Result<(), MeasuredBudgetError<R, Q>>
where
    R: Clone,
    Q: ResourceQuantity,
{
    match limit {
        Some(limit) => limit.check(actual).map_err(MeasuredBudgetError::from),
        None => Ok(()),
    }
}
