// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tracks mutable accounting for one JSON encoding operation.

use super::JsonEncodeAttempt;
use super::JsonEncodeLimits;
use super::internal::EncodeStorage;
use crate::json::JsonResource;
use crate::json::JsonValueBudget;
use crate::resource::ResourceBudget;
use crate::resource::ResourceQuantity;

/// Mutable resource accounting for one JSON encoding operation.
///
/// Use [`Self::from_limits`] for a session that owns budgets created from
/// immutable limits, or [`Self::borrowing_output`] when the caller owns the
/// output and value budgets. Accepted output bytes are charged immediately;
/// value measurements are staged until [`JsonEncodeAttempt::commit`].
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource identity retained by limits and errors.
/// * `Q` - Exact unsigned quantity used for measurements and accounting.
///
/// # Examples
///
/// ```
/// use qubit_budget::json::JsonEncodeLimits;
/// use qubit_budget::json::JsonEncodeSession;
/// use qubit_budget::json::JsonMeasurement;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let limits = JsonEncodeLimits::builder()
///     .max_output_bytes(4_usize)
///     .max_nodes(1_usize)
///     .build();
/// let mut session = JsonEncodeSession::from_limits(limits);
/// let mut attempt = session.begin_value();
/// attempt
///     .try_consume_output_bytes(4)
///     .expect("the output should fit");
/// attempt
///     .try_admit(JsonMeasurement::Null { depth: 1 })
///     .expect("the value should fit");
/// attempt.commit()?;
/// assert_eq!(session.output_budget().expect("output budget").used(), 4);
/// # Ok(()) }
/// ```
#[derive(Debug)]
pub struct JsonEncodeSession<'a, R = JsonResource, Q = usize>
where
    Q: ResourceQuantity,
{
    /// Owned or borrowed budgets backing this encode operation.
    storage: EncodeStorage<'a, R, Q>,
}

impl<'a, R, Q> JsonEncodeSession<'a, R, Q>
where
    R: Clone,
    Q: ResourceQuantity,
{
    /// Creates a session borrowing only a caller-owned value budget.
    ///
    /// # Parameters
    ///
    /// * `value` - Caller-owned JSON value budget to update after a successful
    ///   encode attempt.
    ///
    /// # Returns
    ///
    /// Creates a session borrowing only a caller-owned value budget.
    #[inline]
    #[must_use]
    pub fn borrowing_value(value: &'a mut JsonValueBudget<R, Q>) -> Self {
        Self {
            storage: EncodeStorage::Borrowed { output: None, value },
        }
    }

    /// Creates a session borrowing caller-owned output and value budgets.
    ///
    /// # Parameters
    ///
    /// * `output` - Output supplied to this operation.
    /// * `value` - Caller-owned JSON value budget to update after a successful
    ///   encode attempt.
    ///
    /// # Returns
    ///
    /// Creates a session borrowing caller-owned output and value budgets.
    #[inline]
    #[must_use]
    pub fn borrowing_output(output: &'a mut ResourceBudget<R, Q>, value: &'a mut JsonValueBudget<R, Q>) -> Self {
        Self {
            storage: EncodeStorage::Borrowed {
                output: Some(output),
                value,
            },
        }
    }

    /// Starts accounting for one complete JSON value.
    ///
    /// The returned attempt charges accepted output immediately, but publishes
    /// staged JSON value accounting only after a successful `commit`. A
    /// value-admission failure poisons commit; dropping the attempt rolls back
    /// only the staged value state.
    ///
    /// # Returns
    ///
    /// Starts accounting for one complete JSON value.
    #[must_use]
    pub fn begin_value(&mut self) -> JsonEncodeAttempt<'_, R, Q> {
        let (output, value) = self.storage.split();
        JsonEncodeAttempt::new(output, value.transaction())
    }

    /// Returns the output budget when configured.
    ///
    /// # Returns
    ///
    /// Returns the output budget when configured.
    ///
    /// `None` indicates that the corresponding limit or budget dimension is
    /// unconfigured.
    #[must_use]
    #[inline(always)]
    pub fn output_budget(&self) -> Option<&ResourceBudget<R, Q>> {
        match &self.storage {
            EncodeStorage::Owned { output, .. } => output.as_ref(),
            EncodeStorage::Borrowed { output, .. } => output.as_deref(),
        }
    }

    /// Returns the configured output-byte maximum.
    ///
    /// # Returns
    ///
    /// Returns the configured output-byte maximum.
    ///
    /// `None` indicates that the corresponding limit or budget dimension is
    /// unconfigured.
    #[must_use]
    #[inline(always)]
    pub fn max_output_bytes(&self) -> Option<Q> {
        self.output_budget().map(ResourceBudget::limit)
    }

    /// Returns the value budget for read-only inspection.
    ///
    /// # Returns
    ///
    /// Returns the value budget for read-only inspection.
    #[must_use]
    #[inline(always)]
    pub fn value_budget(&self) -> &JsonValueBudget<R, Q> {
        match &self.storage {
            EncodeStorage::Owned { value, .. } => value,
            EncodeStorage::Borrowed { value, .. } => value,
        }
    }
}

impl<R, Q> JsonEncodeSession<'static, R, Q>
where
    R: Clone,
    Q: ResourceQuantity,
{
    /// Creates a session that owns budgets initialized from immutable limits.
    ///
    /// # Parameters
    ///
    /// * `limits` - Immutable encoding limits used to initialize owned
    ///   accounting budgets.
    ///
    /// # Returns
    ///
    /// Creates a session that owns budgets initialized from immutable limits.
    #[inline]
    #[must_use]
    pub fn from_limits(limits: JsonEncodeLimits<R, Q>) -> Self {
        let output = limits.output_bytes_limit().cloned().map(ResourceBudget::from_limit);
        let value = JsonValueBudget::new(limits.into_value_limits());
        Self {
            storage: EncodeStorage::Owned { output, value },
        }
    }
}
