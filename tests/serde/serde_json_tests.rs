// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================
//! Tests for the generic JSON/Serde budget adapters.

use std::cell::Cell;
use std::collections::BTreeMap;
use std::error::Error;

use ::serde::Deserialize;
use ::serde::Serialize;
use ::serde::Serializer;
use ::serde::de::DeserializeSeed;
use ::serde::de::Deserializer;
use ::serde::de::IgnoredAny;
use ::serde::ser::SerializeMap;
use ::serde::ser::SerializeSeq;
use ::serde_json::Number;
use ::serde_json::Value;
use qubit_budget::BudgetError;
use qubit_budget::JsonLimits;
use qubit_budget::JsonResource;
use qubit_budget::JsonSerdeError;
use qubit_budget::ResourcePool;
use qubit_budget::ResourceReleaseError;
use qubit_budget::from_slice_seed_with_budget;
use qubit_budget::from_slice_with_budget;
use qubit_budget::to_vec_with_budget;
use qubit_budget::to_writer_with_budget;

/// Arbitrary-precision number text emitted through serde_json's private token.
const LARGE_NUMBER_TEXT: &str = "123456789012345678901234567890";

/// Asserts input preflight and output serialization agree at one limit boundary.
///
/// Both paths must accept `accepted` and reject `rejected` with `expected`.
fn assert_input_output_limit_boundary<T>(
    input: &[u8],
    value: &T,
    accepted: JsonLimits,
    rejected: JsonLimits,
    expected: JsonResource,
) where
    T: Serialize + ?Sized,
{
    let mut input_budget = accepted.clone().budget();
    from_slice_with_budget::<IgnoredAny, _>(input, &mut input_budget)
        .expect("input preflight must accept the exact limit");

    let mut output_budget = accepted.budget();
    let output = to_vec_with_budget(value, &mut output_budget)
        .expect("output serialization must accept the exact limit");
    assert_eq!(
        output, input,
        "input and output checks must use the same compact JSON document"
    );

    let mut input_budget = rejected.clone().budget();
    let input_error = from_slice_with_budget::<IgnoredAny, _>(input, &mut input_budget)
        .expect_err("input preflight must reject one byte below the limit");
    let JsonSerdeError::Budget(input_error) = input_error else {
        panic!("expected an input budget error, got {input_error:?}");
    };
    assert_eq!(input_error.resource(), &expected);

    let mut output_budget = rejected.budget();
    let output_error = to_vec_with_budget(value, &mut output_budget)
        .expect_err("output serialization must reject one byte below the limit");
    let JsonSerdeError::Budget(output_error) = output_error else {
        panic!("expected an output budget error, got {output_error:?}");
    };
    assert_eq!(output_error.resource(), &expected);
}

/// Asserts a budget violation interrupts serialization before its trailing value.
fn assert_online_rejection<T>(
    value: &T,
    limits: JsonLimits,
    expected: JsonResource,
    serialized_tail: &Cell<usize>,
) where
    T: Serialize + ?Sized,
{
    let mut budget = limits.budget();
    let error = to_vec_with_budget(value, &mut budget)
        .expect_err("the online serializer must reject the first value");
    let JsonSerdeError::Budget(error) = error else {
        panic!("expected a budget error, got {error:?}");
    };
    assert_eq!(error.resource(), &expected);
    assert_eq!(
        serialized_tail.get(),
        0,
        "the online serializer must stop before the trailing value"
    );
}

/// Asserts a value and its compact JSON serialization share one limit boundary.
fn assert_serialized_input_output_limit_boundary<T>(
    value: &T,
    accepted: JsonLimits,
    rejected: JsonLimits,
    expected: JsonResource,
) where
    T: Serialize + ?Sized,
{
    let input = serde_json::to_vec(value).expect("fixture must serialize as JSON");
    assert_input_output_limit_boundary(&input, value, accepted, rejected, expected);
}

#[derive(Debug, Deserialize, PartialEq)]
struct Borrowed<'a> {
    #[serde(borrow)]
    value: &'a str,
}

struct CountedSequence<'a> {
    serialized: &'a Cell<usize>,
    len: usize,
}

impl Serialize for CountedSequence<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(Some(self.len))?;
        for value in 0..self.len {
            self.serialized.set(self.serialized.get() + 1);
            sequence.serialize_element(&value)?;
        }
        sequence.end()
    }
}

struct CountedDepth<'a> {
    serialized: &'a Cell<usize>,
    remaining: usize,
}

/// Sequence containing one budgeted value before an observable trailing value.
struct SequenceThenTail<'a, T: ?Sized> {
    /// Value whose budget rejection must stop traversal.
    first: &'a T,

    /// Number of times traversal reached the trailing value.
    serialized_tail: &'a Cell<usize>,
}

impl<T> Serialize for SequenceThenTail<'_, T>
where
    T: Serialize + ?Sized,
{
    /// Serializes the first value, then records entry into a trailing null.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(Some(2))?;
        sequence.serialize_element(self.first)?;
        self.serialized_tail.set(self.serialized_tail.get() + 1);
        sequence.serialize_element(&())?;
        sequence.end()
    }
}

/// Map containing one budgeted entry before an observable trailing entry.
struct MapThenTail<'a, K: ?Sized, V: ?Sized> {
    /// Key whose serialization may exhaust the key budget.
    key: &'a K,

    /// Value whose serialization may exhaust a nested container budget.
    value: &'a V,

    /// Number of times traversal reached the trailing entry.
    serialized_tail: &'a Cell<usize>,
}

impl<K, V> Serialize for MapThenTail<'_, K, V>
where
    K: Serialize + ?Sized,
    V: Serialize + ?Sized,
{
    /// Serializes the first entry, then records entry into a trailing null.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry(self.key, self.value)?;
        self.serialized_tail.set(self.serialized_tail.get() + 1);
        map.serialize_entry("tail", &())?;
        map.end()
    }
}

/// Arbitrary-precision number followed by an observable sequence tail.
struct NumberThenTail<'a> {
    /// Arbitrary-precision number emitted before the tail.
    number: Number,

    /// Number of times traversal reached the trailing value.
    serialized_tail: &'a Cell<usize>,
}

impl<'a> NumberThenTail<'a> {
    /// Creates an arbitrary-precision number fixture with an observable tail.
    fn new(serialized_tail: &'a Cell<usize>) -> Self {
        let number = LARGE_NUMBER_TEXT
            .parse::<Number>()
            .expect("the arbitrary-precision number fixture must parse");
        Self {
            number,
            serialized_tail,
        }
    }
}

impl Serialize for NumberThenTail<'_> {
    /// Serializes the arbitrary-precision number before the trailing null.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SequenceThenTail {
            first: &self.number,
            serialized_tail: self.serialized_tail,
        }
        .serialize(serializer)
    }
}

/// Sequence that intentionally omits its length hint.
struct UnknownSequence(usize);

impl Serialize for UnknownSequence {
    /// Serializes every integer without declaring the sequence length.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(None)?;
        for value in 0..self.0 {
            sequence.serialize_element(&value)?;
        }
        sequence.end()
    }
}

/// Map that intentionally omits its length hint.
struct UnknownMap(usize);

impl Serialize for UnknownMap {
    /// Serializes integer keys and values without declaring the map length.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        for value in 0..self.0 {
            map.serialize_entry(&value, &value)?;
        }
        map.end()
    }
}

/// Enum shapes emitted through serde_json's default externally tagged layout.
#[derive(Serialize)]
enum EnumShape {
    /// Unit variant represented as a JSON string.
    Unit,

    /// Newtype variant represented as a one-entry object.
    Newtype(u8),

    /// Tuple variant represented as an object containing an array.
    Tuple(u8, u8),

    /// Struct variant represented as nested objects.
    Struct {
        /// First value encoded under the inner object.
        first: u8,

        /// Second value encoded under the inner object.
        second: u8,
    },
}

impl Serialize for CountedDepth<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.serialized.set(self.serialized.get() + 1);
        if self.remaining == 0 {
            serializer.serialize_unit()
        } else {
            let mut sequence = serializer.serialize_seq(Some(1))?;
            sequence.serialize_element(&CountedDepth {
                serialized: self.serialized,
                remaining: self.remaining - 1,
            })?;
            sequence.end()
        }
    }
}

#[test]
fn test_borrowed_decode_charges_structure_and_preserves_borrowing() {
    let mut budget = JsonLimits::new()
        .with_max_input_bytes(32)
        .with_max_nodes(2)
        .with_max_map_entries(1)
        .with_max_key_bytes(5)
        .with_max_string_bytes(5)
        .budget();
    let input = br#"{"value":"hello"}"#;
    let decoded: Borrowed<'_> =
        from_slice_with_budget(input, &mut budget).unwrap();
    assert_eq!(decoded, Borrowed { value: "hello" });
    assert!(std::ptr::eq(decoded.value.as_ptr(), input[10..15].as_ptr()));
}

#[test]
fn test_input_limit_is_checked_before_json_parsing() {
    let mut budget = JsonLimits::new().with_max_input_bytes(2).budget();
    let error =
        from_slice_with_budget::<IgnoredAny, _>(b"not json", &mut budget)
            .expect_err("input bytes must be rejected before syntax parsing");
    assert!(matches!(
        error,
        JsonSerdeError::Budget(BudgetError::LimitExceeded {
            resource: JsonResource::InputBytes,
            actual: 8,
            maximum: 2,
        })
    ));
}

#[test]
fn test_depth_and_string_limits_are_reported_as_budget_sources() {
    let mut budget = JsonLimits::new()
        .with_max_depth(1)
        .with_max_string_bytes(2)
        .budget();
    let error =
        from_slice_with_budget::<Value, _>(br#"{"x":"long"}"#, &mut budget)
            .expect_err("the key or nested value should exceed a limit");
    assert!(matches!(error, JsonSerdeError::Budget(_)));
    assert!(error.source().is_some());
}

#[test]
fn test_trailing_json_is_rejected() {
    let mut budget = JsonLimits::new().budget();
    let error = from_slice_with_budget::<bool, _>(b"true false", &mut budget)
        .expect_err("trailing content must be rejected");
    assert!(matches!(error, JsonSerdeError::Json(_)));
}

struct IgnoreSeed;

impl<'de> DeserializeSeed<'de> for IgnoreSeed {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        IgnoredAny::deserialize(deserializer).map(|_| ())
    }
}

#[test]
fn test_custom_seed_uses_the_same_budget_session() {
    let mut budget = JsonLimits::new().with_max_nodes(1).budget();
    let error = from_slice_seed_with_budget(b"[null]", IgnoreSeed, &mut budget)
        .expect_err("array and child need two nodes");
    assert!(matches!(error, JsonSerdeError::Budget(_)));
}

#[test]
fn test_output_limit_is_checked_before_writer_io() {
    let mut budget = JsonLimits::new().with_max_output_bytes(2).budget();
    let mut output = Vec::new();
    let error = to_writer_with_budget(&mut output, &"hello", &mut budget)
        .expect_err("serialized output should exceed the limit");
    assert!(matches!(error, JsonSerdeError::Budget(_)));
    assert!(output.is_empty());
}

#[test]
fn test_json_output_writer_rejects_before_vec_growth() {
    let serialized = Cell::new(0);
    let value = CountedSequence {
        serialized: &serialized,
        len: 1_000,
    };
    let mut budget = JsonLimits::new().with_max_output_bytes(8).budget();

    let error = to_vec_with_budget(&value, &mut budget)
        .expect_err("the ninth output byte must be rejected");

    assert!(matches!(
        error,
        JsonSerdeError::Budget(BudgetError::LimitExceeded {
            resource: JsonResource::OutputBytes,
            actual: 9,
            maximum: 8,
        })
    ));
    assert!(serialized.get() < 1_000);
}

#[test]
fn test_output_budget_keeps_destination_writer_empty() {
    let mut budget = JsonLimits::new().with_max_output_bytes(8).budget();
    let mut output = Vec::new();

    let error = to_writer_with_budget(&mut output, &vec![1_u8; 1_000], &mut budget)
        .expect_err("the output budget must reject the value");

    assert!(matches!(error, JsonSerdeError::Budget(_)));
    assert!(output.is_empty());
}

#[test]
fn test_output_node_budget_stops_serialize_before_source_is_exhausted() {
    let serialized = Cell::new(0);
    let value = CountedSequence {
        serialized: &serialized,
        len: 1_000,
    };
    let mut budget = JsonLimits::new().with_max_nodes(3).budget();

    let error =
        to_vec_with_budget(&value, &mut budget).expect_err("node budget must stop serialization");

    assert!(matches!(error, JsonSerdeError::Budget(_)));
    assert!(serialized.get() < 1_000);
}

#[test]
fn test_output_depth_budget_stops_before_full_recursive_serialize() {
    const SOURCE_DEPTH: usize = 128;

    let serialized = Cell::new(0);
    let value = CountedDepth {
        serialized: &serialized,
        remaining: SOURCE_DEPTH - 1,
    };
    let mut budget = JsonLimits::new().with_max_depth(4).budget();

    let error = to_vec_with_budget(&value, &mut budget)
        .expect_err("depth budget must stop recursive serialization");

    assert!(matches!(
        error,
        JsonSerdeError::Budget(BudgetError::LimitExceeded {
            resource: JsonResource::Depth,
            actual: 5,
            maximum: 4,
        })
    ));
    assert!(serialized.get() < SOURCE_DEPTH);
}

#[test]
fn test_output_vector_and_writer_success() {
    let mut vector_budget = JsonLimits::new().with_max_output_bytes(7).budget();
    let bytes = to_vec_with_budget(&[1_u8, 2_u8], &mut vector_budget).unwrap();
    assert_eq!(bytes, b"[1,2]");

    let mut writer_budget = JsonLimits::new().with_max_output_bytes(7).budget();
    let mut output = Vec::new();
    to_writer_with_budget(&mut output, &[1_u8, 2_u8], &mut writer_budget)
        .unwrap();
    assert_eq!(output, bytes);
}

/// Verifies arbitrary-precision numbers have identical input and output bounds.
#[test]
fn test_arbitrary_precision_number_input_output_limit_boundary() {
    let value = LARGE_NUMBER_TEXT
        .parse::<Number>()
        .expect("the arbitrary-precision number fixture must parse");
    let accepted = JsonLimits::new().with_max_number_bytes(LARGE_NUMBER_TEXT.len());
    let rejected = JsonLimits::new().with_max_number_bytes(LARGE_NUMBER_TEXT.len() - 1);

    assert_input_output_limit_boundary(
        LARGE_NUMBER_TEXT.as_bytes(),
        &value,
        accepted,
        rejected,
        JsonResource::NumberBytes,
    );
}

/// Verifies escaped Unicode object keys and strings use decoded UTF-8 lengths.
#[test]
fn test_escaped_unicode_input_output_limit_boundaries() {
    let input = r#"{"a\n\"你":"x\t好"}"#;
    let value = BTreeMap::from([("a\n\"你", "x\t好")]);
    let key_bytes = "a\n\"你".len();
    let string_bytes = "x\t好".len();

    assert_input_output_limit_boundary(
        input.as_bytes(),
        &value,
        JsonLimits::new()
            .with_max_key_bytes(key_bytes)
            .with_max_string_bytes(string_bytes),
        JsonLimits::new()
            .with_max_key_bytes(key_bytes - 1)
            .with_max_string_bytes(string_bytes),
        JsonResource::KeyBytes,
    );
    assert_input_output_limit_boundary(
        input.as_bytes(),
        &value,
        JsonLimits::new()
            .with_max_key_bytes(key_bytes)
            .with_max_string_bytes(string_bytes),
        JsonLimits::new()
            .with_max_key_bytes(key_bytes)
            .with_max_string_bytes(string_bytes - 1),
        JsonResource::StringBytes,
    );
}

/// Verifies integer map keys use their emitted JSON key text in both paths.
#[test]
fn test_integer_map_key_input_output_limit_boundary() {
    let input = br#"{"-12":true}"#;
    let value = BTreeMap::from([(-12_i32, true)]);

    assert_input_output_limit_boundary(
        input,
        &value,
        JsonLimits::new().with_max_key_bytes(3),
        JsonLimits::new().with_max_key_bytes(2),
        JsonResource::KeyBytes,
    );
}

/// Verifies every enum shape has matching input and output limit boundaries.
#[test]
fn test_enum_input_output_limit_boundaries() {
    assert_serialized_input_output_limit_boundary(
        &EnumShape::Unit,
        JsonLimits::new().with_max_string_bytes(4),
        JsonLimits::new().with_max_string_bytes(3),
        JsonResource::StringBytes,
    );
    assert_serialized_input_output_limit_boundary(
        &EnumShape::Newtype(1),
        JsonLimits::new().with_max_map_entries(1),
        JsonLimits::new().with_max_map_entries(0),
        JsonResource::MapEntries,
    );
    assert_serialized_input_output_limit_boundary(
        &EnumShape::Tuple(1, 2),
        JsonLimits::new().with_max_sequence_items(2),
        JsonLimits::new().with_max_sequence_items(1),
        JsonResource::SequenceItems,
    );
    assert_serialized_input_output_limit_boundary(
        &EnumShape::Struct {
            first: 3,
            second: 4,
        },
        JsonLimits::new().with_max_map_entries(2),
        JsonLimits::new().with_max_map_entries(1),
        JsonResource::MapEntries,
    );
}

/// Verifies unknown-length containers have matching input and output bounds.
#[test]
fn test_unknown_container_input_output_limit_boundaries() {
    assert_serialized_input_output_limit_boundary(
        &UnknownSequence(2),
        JsonLimits::new().with_max_sequence_items(2),
        JsonLimits::new().with_max_sequence_items(1),
        JsonResource::SequenceItems,
    );
    assert_serialized_input_output_limit_boundary(
        &UnknownMap(2),
        JsonLimits::new().with_max_map_entries(2),
        JsonLimits::new().with_max_map_entries(1),
        JsonResource::MapEntries,
    );
}

/// Verifies an arbitrary-precision number is rejected before trailing traversal.
#[test]
fn test_online_serializer_rejects_arbitrary_precision_number_before_tail() {
    let tail = Cell::new(0);
    let value = NumberThenTail::new(&tail);
    assert_online_rejection(
        &value,
        JsonLimits::new().with_max_number_bytes(LARGE_NUMBER_TEXT.len() - 1),
        JsonResource::NumberBytes,
        &tail,
    );
}

/// Verifies escaped Unicode keys reject before serializing a following entry.
#[test]
fn test_online_serializer_rejects_escaped_unicode_key_before_tail() {
    let tail = Cell::new(0);
    let key = "a\n\"你";
    let value = MapThenTail {
        key: &key,
        value: &(),
        serialized_tail: &tail,
    };
    assert_online_rejection(
        &value,
        JsonLimits::new().with_max_key_bytes(key.len() - 1),
        JsonResource::KeyBytes,
        &tail,
    );
}

/// Verifies escaped Unicode strings reject before serializing a following value.
#[test]
fn test_online_serializer_rejects_escaped_unicode_string_before_tail() {
    let tail = Cell::new(0);
    let string = "x\t好";
    let value = SequenceThenTail {
        first: &string,
        serialized_tail: &tail,
    };
    assert_online_rejection(
        &value,
        JsonLimits::new().with_max_string_bytes(string.len() - 1),
        JsonResource::StringBytes,
        &tail,
    );
}

/// Verifies integer map keys reject before serializing a following entry.
#[test]
fn test_online_serializer_rejects_integer_map_key_before_tail() {
    let tail = Cell::new(0);
    let key = -12_i32;
    let value = MapThenTail {
        key: &key,
        value: &(),
        serialized_tail: &tail,
    };
    assert_online_rejection(
        &value,
        JsonLimits::new().with_max_key_bytes(2),
        JsonResource::KeyBytes,
        &tail,
    );
}

/// Verifies all enum shapes reject before serializing a following value.
#[test]
fn test_online_serializer_rejects_enum_shapes_before_tail() {
    let tail = Cell::new(0);
    let unit = SequenceThenTail {
        first: &EnumShape::Unit,
        serialized_tail: &tail,
    };
    assert_online_rejection(
        &unit,
        JsonLimits::new().with_max_string_bytes(3),
        JsonResource::StringBytes,
        &tail,
    );

    let tail = Cell::new(0);
    let newtype = SequenceThenTail {
        first: &EnumShape::Newtype(1),
        serialized_tail: &tail,
    };
    assert_online_rejection(
        &newtype,
        JsonLimits::new().with_max_map_entries(0),
        JsonResource::MapEntries,
        &tail,
    );

    let tail = Cell::new(0);
    let tuple = MapThenTail {
        key: &"first",
        value: &EnumShape::Tuple(1, 2),
        serialized_tail: &tail,
    };
    assert_online_rejection(
        &tuple,
        JsonLimits::new().with_max_sequence_items(1),
        JsonResource::SequenceItems,
        &tail,
    );

    let tail = Cell::new(0);
    let structure = SequenceThenTail {
        first: &EnumShape::Struct {
            first: 3,
            second: 4,
        },
        serialized_tail: &tail,
    };
    assert_online_rejection(
        &structure,
        JsonLimits::new().with_max_map_entries(1),
        JsonResource::MapEntries,
        &tail,
    );
}

/// Verifies an unknown sequence rejects at its exact online boundary before tail.
#[test]
fn test_online_serializer_rejects_unknown_sequence_boundary_before_tail() {
    let tail = Cell::new(0);
    let sequence = UnknownSequence(2);
    let value = MapThenTail {
        key: &"first",
        value: &sequence,
        serialized_tail: &tail,
    };
    assert_online_rejection(
        &value,
        JsonLimits::new().with_max_sequence_items(1),
        JsonResource::SequenceItems,
        &tail,
    );
}

/// Verifies an unknown map rejects at its exact online boundary before tail.
#[test]
fn test_online_serializer_rejects_unknown_map_boundary_before_tail() {
    let tail = Cell::new(0);
    let map = UnknownMap(2);
    let value = SequenceThenTail {
        first: &map,
        serialized_tail: &tail,
    };
    assert_online_rejection(
        &value,
        JsonLimits::new().with_max_map_entries(1),
        JsonResource::MapEntries,
        &tail,
    );
}

#[test]
fn test_release_error_is_not_a_budget_error() {
    let mut pool = ResourcePool::new(JsonResource::Nodes, 1_usize);
    let error = pool
        .release(1)
        .expect_err("an unused unit cannot be released");
    assert!(matches!(error, ResourceReleaseError::InvalidRelease { .. }));
}
