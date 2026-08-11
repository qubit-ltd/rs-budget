// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================
//! Tests for lexical JSON preflight resource accounting.

use qubit_budget::BudgetError;
use qubit_budget::JsonLimits;
use qubit_budget::JsonResource;
use qubit_budget::JsonSerdeError;
use qubit_budget::from_slice_with_budget;
use serde::de::IgnoredAny;

/// Private token used by serde_json for arbitrary-precision numbers.
const JSON_NUMBER_TOKEN: &str = concat!("$", "serde_json", ":", ":private::Number");

/// Asserts that preflight reports one exact point-limit violation.
fn assert_input_limit(
    input: &str,
    limits: JsonLimits,
    expected: JsonResource,
    actual: usize,
    maximum: usize,
) {
    let mut budget = limits.budget();
    let error = from_slice_with_budget::<IgnoredAny, _>(input.as_bytes(), &mut budget)
        .expect_err("the configured resource limit must reject the input");
    let description = format!("{error:?}");
    assert!(
        matches!(
            error,
            JsonSerdeError::Budget(BudgetError::LimitExceeded {
                resource,
                actual: error_actual,
                maximum: error_maximum,
            }) if resource == expected && error_actual == actual && error_maximum == maximum
        ),
        "unexpected error: {description}"
    );
}

/// Asserts that preflight exhausts the exact configured node budget.
fn assert_input_nodes(input: &str, maximum: usize) {
    let mut budget = JsonLimits::new().with_max_nodes(maximum).budget();
    let error = from_slice_with_budget::<IgnoredAny, _>(input.as_bytes(), &mut budget)
        .expect_err("the configured node budget must reject the input");
    assert!(matches!(
        error,
        JsonSerdeError::Budget(BudgetError::Insufficient {
            resource: JsonResource::Nodes,
            limit,
            remaining: 0,
            requested: 1,
        }) if limit == maximum
    ));
}

/// Verifies that lexical preflight charges the UTF-8 length of string values.
#[test]
fn test_json_lexical_preflight_charges_string_bytes() {
    let mut budget = JsonLimits::new().with_max_string_bytes(3).budget();
    let error = from_slice_with_budget::<String, _>(br#""hello""#, &mut budget)
        .expect_err("the string should exceed the byte budget");

    assert!(matches!(
        error,
        JsonSerdeError::Budget(BudgetError::LimitExceeded {
            resource: JsonResource::StringBytes,
            actual: 5,
            maximum: 3,
        })
    ));
}

/// Verifies a colliding private token with another field remains an object.
#[test]
fn test_json_lexical_preflight_charges_colliding_number_token_as_object() {
    let input = format!(r#"{{"{JSON_NUMBER_TOKEN}":"1","x":2}}"#);
    let mut budget = JsonLimits::new().with_max_key_bytes(0).budget();

    let error = from_slice_with_budget::<IgnoredAny, _>(input.as_bytes(), &mut budget)
        .expect_err("the first ordinary object key must consume the key budget");

    assert!(matches!(
        error,
        JsonSerdeError::Budget(BudgetError::LimitExceeded {
            resource: JsonResource::KeyBytes,
            actual,
            maximum: 0,
        }) if actual == JSON_NUMBER_TOKEN.len()
    ));
}

/// Verifies a single-field ordinary object cannot impersonate a Number token.
#[test]
fn test_json_lexical_preflight_charges_single_number_token_object() {
    let input = format!(r#"{{"{JSON_NUMBER_TOKEN}":"x"}}"#);
    let mut budget = JsonLimits::new().with_max_key_bytes(0).budget();

    let error = from_slice_with_budget::<IgnoredAny, _>(input.as_bytes(), &mut budget)
        .expect_err("a textual private-token field is still an ordinary object key");

    assert!(matches!(
        error,
        JsonSerdeError::Budget(BudgetError::LimitExceeded {
            resource: JsonResource::KeyBytes,
            actual,
            maximum: 0,
        }) if actual == JSON_NUMBER_TOKEN.len()
    ));
}

/// Verifies a numeric first value under the private token remains an object.
#[test]
fn test_json_lexical_preflight_accepts_colliding_token_with_numeric_value() {
    let input = format!(r#"{{"{JSON_NUMBER_TOKEN}":1,"x":2}}"#);
    let mut budget = JsonLimits::new().budget();

    from_slice_with_budget::<IgnoredAny, _>(input.as_bytes(), &mut budget)
        .expect("a numeric private-token field with a sibling is a valid object");
}

/// Verifies a nested object under the private token remains an object field.
#[test]
fn test_json_lexical_preflight_accepts_colliding_token_with_nested_object() {
    let input = format!(r#"{{"{JSON_NUMBER_TOKEN}":{{"nested":true}},"x":2}}"#);
    let mut budget = JsonLimits::new().budget();

    from_slice_with_budget::<IgnoredAny, _>(input.as_bytes(), &mut budget)
        .expect("a nested object under the private token is a valid field");
}

/// Verifies a nested array under the private token remains an object field.
#[test]
fn test_json_lexical_preflight_accepts_colliding_token_with_nested_array() {
    let input = format!(r#"{{"{JSON_NUMBER_TOKEN}":[null],"x":2}}"#);
    let mut budget = JsonLimits::new().budget();

    from_slice_with_budget::<IgnoredAny, _>(input.as_bytes(), &mut budget)
        .expect("a nested array under the private token is a valid field");
}

/// Verifies colliding-token objects retain depth, node, and string budgets.
#[test]
fn test_json_lexical_preflight_charges_colliding_object_value_resources() {
    let nested = format!(r#"{{"{JSON_NUMBER_TOKEN}":{{"nested":true}},"x":2}}"#);
    assert_input_limit(
        &nested,
        JsonLimits::new().with_max_depth(2),
        JsonResource::Depth,
        3,
        2,
    );
    assert_input_nodes(&nested, 2);

    let string = format!(r#"{{"{JSON_NUMBER_TOKEN}":"long","x":2}}"#);
    assert_input_limit(
        &string,
        JsonLimits::new().with_max_string_bytes(3),
        JsonResource::StringBytes,
        4,
        3,
    );
}

/// Verifies duplicate private-token fields remain two ordinary object entries.
#[test]
fn test_json_lexical_preflight_charges_duplicate_number_token_entries() {
    let input = format!(r#"{{"{JSON_NUMBER_TOKEN}":"1","{JSON_NUMBER_TOKEN}":"2"}}"#);
    let mut budget = JsonLimits::new().with_max_map_entries(1).budget();

    let error = from_slice_with_budget::<IgnoredAny, _>(input.as_bytes(), &mut budget)
        .expect_err("duplicate object fields must consume separate map entries");

    assert!(matches!(
        error,
        JsonSerdeError::Budget(BudgetError::LimitExceeded {
            resource: JsonResource::MapEntries,
            actual: 2,
            maximum: 1,
        })
    ));
}
