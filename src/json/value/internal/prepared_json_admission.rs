// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Prepares native JSON measurements for deterministic budget admission.

use super::super::JsonValueLimits;
use crate::json::JsonMeasurement;
use crate::resource::MeasuredBudgetError;
use crate::resource::ResourceLimit;
use crate::resource::ResourceQuantity;

/// Native JSON measurement converted for the dimensions configured by limits.
///
/// # Type Parameters
///
/// * `Q` - Exact unsigned quantity used for measurements and accounting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::json) enum PreparedJsonAdmission<Q>
where
    Q: ResourceQuantity,
{
    /// A null value with its converted depth.
    Null {
        /// Root-inclusive nesting depth used for the point check.
        depth: Q,
    },
    /// A boolean value with its converted depth.
    Boolean {
        /// Root-inclusive nesting depth used for the point check.
        depth: Q,
    },
    /// A string value with its converted depth and byte length.
    String {
        /// Root-inclusive nesting depth used for the point check.
        depth: Q,
        /// UTF-8 byte length used for point and cumulative checks.
        bytes: Q,
    },
    /// A number value with its converted depth and byte length.
    Number {
        /// Root-inclusive nesting depth used for the point check.
        depth: Q,
        /// Representation byte length used for point and cumulative checks.
        bytes: Q,
    },
    /// An array with its converted depth and item count.
    Array {
        /// Root-inclusive nesting depth used for the point check.
        depth: Q,
        /// Number of direct array items used for the point check.
        items: Q,
    },
    /// An object with its converted depth and entry count.
    Object {
        /// Root-inclusive nesting depth used for the point check.
        depth: Q,
        /// Number of direct object entries used for the point check.
        entries: Q,
    },
    /// An object key with its converted byte length.
    Key {
        /// UTF-8 byte length used for point and cumulative checks.
        bytes: Q,
    },
}

impl<Q> PreparedJsonAdmission<Q>
where
    Q: ResourceQuantity,
{
    /// Converts a measurement only for dimensions configured by `limits`.
    ///
    /// Returns a conversion error associated with the first configured
    /// resource whose native measurement does not fit `Q`.
    ///
    /// # Type Parameters
    ///
    /// * `R` - Caller-defined resource identity retained by limits and errors.
    ///
    /// # Parameters
    ///
    /// * `limits` - JSON value limits that determine which measurements need
    ///   conversion.
    /// * `measurement` - Native JSON measurement to convert or admit.
    ///
    /// # Returns
    ///
    /// `Ok(prepared)` with every configured native dimension converted to `Q`.
    ///
    /// # Errors
    ///
    /// Returns [`MeasuredBudgetError::Quantity`] when a configured native
    /// measurement cannot fit `Q`. This method does not check point limits.
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
                bytes: convert_payload(bytes, limits.string_bytes_limit(), limits.payload_bytes_limit())?,
            }),
            JsonMeasurement::Number { depth, bytes } => Ok(Self::Number {
                depth: convert(depth, structure.depth_limit())?,
                bytes: convert_payload(bytes, limits.number_bytes_limit(), limits.payload_bytes_limit())?,
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
                bytes: convert_payload(bytes, structure.key_bytes_limit(), limits.payload_bytes_limit())?,
            }),
        }
    }

    /// Checks configured point limits after native measurement conversion.
    ///
    /// Returns the first depth or variant-specific limit error. This method
    /// neither creates nor mutates a budget.
    ///
    /// # Type Parameters
    ///
    /// * `R` - Caller-defined resource identity retained by limits and errors.
    ///
    /// # Parameters
    ///
    /// * `limits` - JSON value limits whose configured point dimensions are
    ///   checked.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the operation completes successfully.
    ///
    /// # Errors
    ///
    /// Returns [`MeasuredBudgetError::Budget`] when a configured point limit
    /// rejects the converted measurement.
    pub(in crate::json) fn check_point<R>(
        &self,
        limits: &JsonValueLimits<R, Q>,
    ) -> Result<(), MeasuredBudgetError<R, Q>>
    where
        R: Clone,
    {
        let structure = limits.structure_limits();
        match self {
            Self::Null { depth } | Self::Boolean { depth } => check_limit(structure.depth_limit(), *depth),
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
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource identity retained by limits and errors.
/// * `Q` - Exact unsigned quantity used for measurements and accounting.
///
/// # Parameters
///
/// * `amount` - Native measurement to convert when `limit` is configured.
/// * `limit` - Optional resource-bound limit that determines the conversion
///   error's resource identity.
///
/// # Returns
///
/// `Ok(converted)` when the limit is configured, or `Ok(Q::ZERO)` when the
/// dimension is unconfigured.
///
/// # Errors
///
/// Returns [`MeasuredBudgetError`] when `amount` cannot be represented by `Q`.
fn convert<R, Q>(amount: usize, limit: Option<&ResourceLimit<R, Q>>) -> Result<Q, MeasuredBudgetError<R, Q>>
where
    R: Clone,
    Q: ResourceQuantity,
{
    let Some(limit) = limit else {
        return Ok(Q::ZERO);
    };
    Q::try_from_usize(amount).map_err(|source| MeasuredBudgetError::quantity(limit.resource().clone(), source))
}

/// Converts a native payload quantity when either point or cumulative limits
/// are set.
///
/// Returns a conversion error carrying the point-limit resource when present,
/// otherwise the cumulative payload-limit resource.
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource identity retained by limits and errors.
/// * `Q` - Exact unsigned quantity used for measurements and accounting.
///
/// # Parameters
///
/// * `amount` - Native payload byte length to convert.
/// * `point_limit` - Optional point limit whose resource takes precedence in
///   conversion errors.
/// * `payload_limit` - Optional cumulative limit used when no point limit is
///   configured.
///
/// # Returns
///
/// `Ok(converted)` when a relevant limit exists, or `Ok(Q::ZERO)` when the
/// byte dimension is entirely unconfigured.
///
/// # Errors
///
/// Returns [`MeasuredBudgetError`] when `amount` cannot be represented by `Q`.
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
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource identity retained by limits and errors.
/// * `Q` - Exact unsigned quantity used for measurements and accounting.
///
/// # Parameters
///
/// * `limit` - Optional point limit to check.
/// * `actual` - Observed quantity to validate.
///
/// # Returns
///
/// `Ok(())` when the operation completes successfully.
///
/// # Errors
///
/// Returns [`MeasuredBudgetError`] when a configured point limit rejects
/// `actual`.
fn check_limit<R, Q>(limit: Option<&ResourceLimit<R, Q>>, actual: Q) -> Result<(), MeasuredBudgetError<R, Q>>
where
    R: Clone,
    Q: ResourceQuantity,
{
    match limit {
        Some(limit) => limit.check(actual).map_err(MeasuredBudgetError::from),
        None => Ok(()),
    }
}
