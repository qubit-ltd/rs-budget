// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for JSON decoding admission and session accounting.

use qubit_budget::JsonDecodeLimits;
use qubit_budget::JsonDecodeSession;
use qubit_budget::JsonResource;
use qubit_budget::JsonSerdeError;
use qubit_budget::JsonValueLimits;
use qubit_budget::ResourceLimit;
use qubit_budget::StructureLimits;
use qubit_budget::decode_slice;
use qubit_budget::decode_slice_seed;
use serde::Deserialize;
use serde::Deserializer;
use serde::de::DeserializeSeed;
use serde::de::IgnoredAny;

/// Verifies escaped and direct Unicode text consume the same decoded payload.
#[test]
fn escaped_and_direct_unicode_charge_equal_decoded_payload() {
    let limits = JsonDecodeLimits::empty().with_value_limits(
        JsonValueLimits::empty()
            .with_payload_bytes_limit(ResourceLimit::new(JsonResource::PayloadBytes, 3)),
    );
    for input in [br#""\u4e2d""#.as_slice(), "\"中\"".as_bytes()] {
        let mut session = JsonDecodeSession::owned(limits);
        assert_eq!(
            decode_slice::<String, _>(input, &mut session)
                .expect("three decoded UTF-8 bytes must fit"),
            "中"
        );
    }
}

/// Verifies lexical admission rejects excessive depth before typed
/// deserialization.
#[test]
fn deeply_nested_input_fails_by_limit_without_stack_overflow() {
    let input = format!("{}0{}", "[".repeat(20_000), "]".repeat(20_000));
    let mut session = JsonDecodeSession::owned(JsonDecodeLimits::empty().with_value_limits(
        JsonValueLimits::empty().with_structure_limits(
            StructureLimits::empty().with_depth_limit(ResourceLimit::new(JsonResource::Depth, 128)),
        ),
    ));

    assert!(matches!(
        decode_slice::<serde_json::Value, _>(input.as_bytes(), &mut session),
        Err(JsonSerdeError::Budget(_))
    ));
}

/// Verifies typed decode failures still consume the attempted input bytes.
#[test]
fn typed_decode_failure_consumes_input_before_the_next_attempt() {
    let mut session = JsonDecodeSession::owned(
        JsonDecodeLimits::empty()
            .with_input_bytes_limit(ResourceLimit::new(JsonResource::InputBytes, 3)),
    );

    assert!(matches!(
        decode_slice::<u8, _>(br#""x""#, &mut session),
        Err(JsonSerdeError::Json(_))
    ));
    assert!(matches!(
        decode_slice::<u8, _>(b"0", &mut session),
        Err(JsonSerdeError::Budget(_))
    ));
}

/// Verifies seed-first decoding uses the same lexical admission path.
#[test]
fn decode_slice_seed_admits_arbitrary_precision_numbers() {
    let input = b"123456789012345678901234567890";
    let mut session = JsonDecodeSession::owned(JsonDecodeLimits::empty().with_value_limits(
        JsonValueLimits::empty().with_number_bytes_limit(ResourceLimit::new(
            JsonResource::NumberBytes,
            u64::try_from(input.len()).unwrap(),
        )),
    ));

    decode_slice_seed(IgnoreSeed, input, &mut session)
        .expect("the exact arbitrary-precision lexical number limit must fit");
}

/// Verifies lexical limits reject input before a seed is invoked.
#[test]
fn point_limit_fails_before_seed_and_keeps_work_charged() {
    let limits = JsonDecodeLimits::empty().with_value_limits(
        JsonValueLimits::empty()
            .with_structure_limits(
                StructureLimits::empty()
                    .with_nodes_limit(ResourceLimit::new(JsonResource::Nodes, 1)),
            )
            .with_string_bytes_limit(ResourceLimit::new(JsonResource::StringBytes, 1)),
    );
    let mut session = JsonDecodeSession::owned(limits);
    let error =
        decode_slice_seed(PanicSeed, br#""ab""#, &mut session).expect_err("string limit must fail");
    assert!(matches!(error, JsonSerdeError::Budget(_)));
    assert!(session.value_budget_mut().enter_node(1).is_err());
}

struct PanicSeed;

impl<'de> DeserializeSeed<'de> for PanicSeed {
    type Value = ();

    fn deserialize<D>(self, _: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        panic!("seed must not run before lexical admission succeeds");
    }
}

/// Seed that accepts one complete value without constructing a typed payload.
struct IgnoreSeed;

impl<'de> DeserializeSeed<'de> for IgnoreSeed {
    type Value = ();

    /// Ignores the admitted JSON value produced by the supplied deserializer.
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        IgnoredAny::deserialize(deserializer).map(|_| ())
    }
}
