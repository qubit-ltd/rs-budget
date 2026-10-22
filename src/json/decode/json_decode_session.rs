// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tracks mutable accounting for one JSON decoding operation.

use super::JsonDecodeAttempt;
use super::JsonDecodeLimits;
use super::internal::DecodeStorage;
use crate::json::JsonResource;
use crate::json::JsonValueBudget;
use crate::resource::ResourceBudget;
use crate::resource::ResourceQuantity;

/// Mutable resource accounting for one JSON decoding operation.
///
/// Use [`Self::from_limits`] for a session that owns budgets created from
/// immutable limits, or one of the `borrowing_*` constructors when the caller
/// owns the budgets. Create an attempt with [`Self::begin_value`] for each
/// complete value; input charges are immediate, while value accounting is
/// committed by [`JsonDecodeAttempt::commit`].
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource identity retained by limits and errors.
/// * `Q` - Exact unsigned quantity used for measurements and accounting.
///
/// # Examples
///
/// ```
/// use qubit_budget::json::JsonDecodeLimits;
/// use qubit_budget::json::JsonDecodeSession;
/// use qubit_budget::json::JsonMeasurement;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let limits = JsonDecodeLimits::builder()
///     .max_input_bytes(4_usize)
///     .max_nodes(1_usize)
///     .build();
/// let mut session = JsonDecodeSession::from_limits(limits);
/// let mut attempt = session.begin_value();
/// attempt
///     .try_consume_input_bytes(4)
///     .expect("the input should fit");
/// attempt
///     .try_admit(JsonMeasurement::Null { depth: 1 })
///     .expect("the value should fit");
/// attempt.commit()?;
/// assert_eq!(session.input_budget().expect("input budget").used(), 4);
/// # Ok(()) }
/// ```
///
/// An owned session owns its budgets, while resource identities may borrow
/// caller data. The session remains bounded by the identity's lifetime.
///
/// ```compile_fail
/// use qubit_budget::json::{JsonDecodeLimits, JsonDecodeSession};
/// use qubit_budget::ResourceLimit;
///
/// fn invalid_session<'a>() -> JsonDecodeSession<'a, &'a str> {
///     let label = String::from("local-input");
///     let limits = JsonDecodeLimits::<&str, usize>::builder()
///         .input_bytes_limit(ResourceLimit::new(label.as_str(), 2))
///         .build();
///     JsonDecodeSession::from_limits(limits)
/// }
/// ```
#[derive(Debug)]
pub struct JsonDecodeSession<'a, R = JsonResource, Q = usize>
where
    Q: ResourceQuantity,
{
    /// Owned or borrowed budgets backing this decode operation.
    storage: DecodeStorage<'a, R, Q>,
}

impl<'a, R, Q> JsonDecodeSession<'a, R, Q>
where
    R: Clone,
    Q: ResourceQuantity,
{
    /// Creates a session borrowing only a caller-owned value budget.
    ///
    /// # Parameters
    ///
    /// * `value` - Caller-owned JSON value budget to update after a successful
    ///   decode attempt.
    ///
    /// # Returns
    ///
    /// Creates a session borrowing only a caller-owned value budget.
    #[inline]
    #[must_use]
    pub fn borrowing_value(value: &'a mut JsonValueBudget<R, Q>) -> Self {
        Self {
            storage: DecodeStorage::Borrowed {
                input: None,
                normalized_input: None,
                value,
            },
        }
    }

    /// Creates a session borrowing caller-owned raw-input and value budgets.
    ///
    /// # Parameters
    ///
    /// * `input` - Caller-owned raw-input budget to charge during this
    ///   operation.
    /// * `value` - Caller-owned JSON value budget to update after a successful
    ///   decode attempt.
    ///
    /// # Returns
    ///
    /// Creates a session borrowing caller-owned raw-input and value budgets.
    #[inline]
    #[must_use]
    pub fn borrowing_input(input: &'a mut ResourceBudget<R, Q>, value: &'a mut JsonValueBudget<R, Q>) -> Self {
        Self {
            storage: DecodeStorage::Borrowed {
                input: Some(input),
                normalized_input: None,
                value,
            },
        }
    }

    /// Creates a session borrowing normalized-input and value budgets only.
    ///
    /// Raw input accounting is unconfigured. The borrowed resource identities
    /// must remain valid while this session is used.
    ///
    /// # Parameters
    ///
    /// * `normalized_input` - Budget charged immediately for normalized bytes.
    /// * `value` - Budget updated only after successful value commit.
    ///
    /// # Returns
    ///
    /// A session with no raw-input budget and exclusive access to the supplied
    /// normalized-input and value budgets.
    ///
    /// # Examples
    ///
    /// ```
    /// use qubit_budget::ResourceBudget;
    /// use qubit_budget::json::JsonDecodeSession;
    /// use qubit_budget::json::JsonMeasurement;
    /// use qubit_budget::json::JsonResource;
    /// use qubit_budget::json::JsonValueLimits;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut normalized = ResourceBudget::new(JsonResource::NormalizedInputBytes, 8_usize);
    /// let mut value = JsonValueLimits::builder().max_nodes(2).build().budget();
    /// {
    ///     let mut session =
    ///         JsonDecodeSession::borrowing_normalized_input(&mut normalized, &mut value);
    ///     assert!(session.input_budget().is_none());
    ///     let mut attempt = session.begin_value();
    ///     attempt.try_consume_normalized_input_bytes(4)?;
    ///     attempt.try_admit(JsonMeasurement::Null { depth: 1 })?;
    ///     attempt.commit()?;
    /// }
    /// assert_eq!(normalized.used(), 4);
    /// assert_eq!(value.used_nodes(), Some(1));
    /// # Ok(()) }
    /// ```
    #[inline]
    #[must_use]
    pub fn borrowing_normalized_input(
        normalized_input: &'a mut ResourceBudget<R, Q>,
        value: &'a mut JsonValueBudget<R, Q>,
    ) -> Self {
        Self {
            storage: DecodeStorage::Borrowed {
                input: None,
                normalized_input: Some(normalized_input),
                value,
            },
        }
    }

    /// Creates a session borrowing all caller-owned decode budgets.
    ///
    /// # Parameters
    ///
    /// * `input` - Caller-owned raw-input budget to charge during this
    ///   operation.
    /// * `normalized_input` - Caller-owned normalized-input budget to charge
    ///   for normalized input.
    /// * `value` - Caller-owned JSON value budget to update after a successful
    ///   decode attempt.
    ///
    /// # Returns
    ///
    /// Creates a session borrowing all caller-owned decode budgets.
    #[inline]
    #[must_use]
    pub fn borrowing_all(
        input: &'a mut ResourceBudget<R, Q>,
        normalized_input: &'a mut ResourceBudget<R, Q>,
        value: &'a mut JsonValueBudget<R, Q>,
    ) -> Self {
        Self {
            storage: DecodeStorage::Borrowed {
                input: Some(input),
                normalized_input: Some(normalized_input),
                value,
            },
        }
    }

    /// Creates a session that owns budgets initialized from immutable limits.
    ///
    /// Resource identities in the limits may borrow caller data. The owned
    /// budgets do not extend the lifetime of those identities.
    ///
    /// # Parameters
    ///
    /// * `limits` - Immutable limits used to initialize owned accounting
    ///   budgets.
    ///
    /// # Returns
    ///
    /// A session owning its budgets and retaining the limits' resource
    /// identities.
    #[inline]
    #[must_use]
    pub fn from_limits(limits: JsonDecodeLimits<R, Q>) -> Self {
        let input = limits.input_bytes_limit().cloned().map(ResourceBudget::from_limit);
        let normalized_input = limits
            .normalized_input_bytes_limit()
            .cloned()
            .map(ResourceBudget::from_limit);
        let value = JsonValueBudget::new(limits.into_value_limits());
        Self {
            storage: DecodeStorage::Owned {
                input,
                normalized_input,
                value,
            },
        }
    }

    /// Starts accounting for one complete JSON value.
    ///
    /// The returned attempt charges raw and normalized input immediately, but
    /// publishes staged JSON value accounting only after a successful `commit`.
    /// A value-admission failure poisons commit; dropping the attempt rolls
    /// back only the staged value state.
    ///
    /// # Returns
    ///
    /// Starts accounting for one complete JSON value.
    #[must_use]
    pub fn begin_value(&mut self) -> JsonDecodeAttempt<'_, R, Q> {
        let (input, normalized_input, value) = self.storage.split();
        JsonDecodeAttempt::new(input, normalized_input, value.transaction())
    }

    /// Returns the raw input budget when configured.
    ///
    /// # Returns
    ///
    /// Returns the raw input budget when configured.
    ///
    /// `None` indicates that the corresponding limit or budget dimension is
    /// unconfigured.
    #[inline]
    pub fn input_budget(&self) -> Option<&ResourceBudget<R, Q>> {
        match &self.storage {
            DecodeStorage::Owned { input, .. } => input.as_ref(),
            DecodeStorage::Borrowed { input, .. } => input.as_deref(),
        }
    }

    /// Returns the configured raw input-byte maximum.
    ///
    /// # Returns
    ///
    /// Returns the configured raw input-byte maximum.
    ///
    /// `None` indicates that the corresponding limit or budget dimension is
    /// unconfigured.
    #[inline]
    pub fn max_input_bytes(&self) -> Option<Q> {
        self.input_budget().map(ResourceBudget::limit)
    }

    /// Returns the configured normalized input-byte maximum.
    ///
    /// # Returns
    ///
    /// Returns the configured normalized input-byte maximum.
    ///
    /// `None` indicates that the corresponding limit or budget dimension is
    /// unconfigured.
    #[inline]
    pub fn max_normalized_input_bytes(&self) -> Option<Q> {
        self.normalized_input_budget().map(ResourceBudget::limit)
    }

    /// Returns the normalized input budget when configured.
    ///
    /// # Returns
    ///
    /// Returns the normalized input budget when configured.
    ///
    /// `None` indicates that the corresponding limit or budget dimension is
    /// unconfigured.
    #[inline]
    pub fn normalized_input_budget(&self) -> Option<&ResourceBudget<R, Q>> {
        match &self.storage {
            DecodeStorage::Owned { normalized_input, .. } => normalized_input.as_ref(),
            DecodeStorage::Borrowed { normalized_input, .. } => normalized_input.as_deref(),
        }
    }

    /// Returns the value budget for read-only inspection.
    ///
    /// # Returns
    ///
    /// Returns the value budget for read-only inspection.
    #[must_use]
    #[inline]
    pub fn value_budget(&self) -> &JsonValueBudget<R, Q> {
        match &self.storage {
            DecodeStorage::Owned { value, .. } => value,
            DecodeStorage::Borrowed { value, .. } => value,
        }
    }
}
