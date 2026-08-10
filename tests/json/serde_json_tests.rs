// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================
//! Tests for the generic JSON/Serde budget adapters.

use std::error::Error;

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
use serde::Deserialize;
use serde::de::DeserializeSeed;
use serde::de::Deserializer;
use serde::de::IgnoredAny;
use serde_json::Value;

#[derive(Debug, Deserialize, PartialEq)]
struct Borrowed<'a> {
    #[serde(borrow)]
    value: &'a str,
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

#[test]
fn test_release_error_is_not_a_budget_error() {
    let mut pool = ResourcePool::new(JsonResource::Nodes, 1_usize);
    let error = pool
        .release(1)
        .expect_err("an unused unit cannot be released");
    assert!(matches!(error, ResourceReleaseError::InvalidRelease { .. }));
}
