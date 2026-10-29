// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
#![no_main]

mod internal;

use internal::json_transaction_model;
use internal::json_transaction_model::ExpectedFailure;
use internal::json_transaction_model::FailureKind;
use internal::json_transaction_model::Limits;
use internal::json_transaction_model::Model;
use libfuzzer_sys::fuzz_target;
use qubit_budget::MeasuredBudgetError;
use qubit_budget::QuantityMeasurement;
use qubit_budget::ResourceLimit;
use qubit_budget::ResourceQuantity;
use qubit_budget::json::JsonContainerKind;
use qubit_budget::json::JsonMeasurement;
use qubit_budget::json::JsonResource;
use qubit_budget::json::JsonValueBudget;
use qubit_budget::json::JsonValueLimits;
use qubit_budget::json::JsonValueTransaction;

const MAX_INPUT_LEN: usize = 4 * 1024;
const LIMIT_COUNT: usize = 8;
const INSTRUCTION_SIZE: usize = 8;

fuzz_target!(|data: &[u8]| {
    let input = &data[..data.len().min(MAX_INPUT_LEN)];
    let wide_quantity = input.first().copied().unwrap_or_default() & 1 != 0;
    let enabled = input.get(1).copied().unwrap_or_default();
    let mut maxima = [None; LIMIT_COUNT];
    for (index, maximum) in maxima.iter_mut().enumerate() {
        if enabled & (1 << index) != 0 {
            *maximum = Some(u128::from(input.get(index + 2).copied().unwrap_or_default()));
        }
    }
    let limits = Limits {
        maxima,
        quantity_maximum: if wide_quantity {
            u64::MAX as u128
        } else {
            u8::MAX as u128
        },
    };
    if wide_quantity {
        run::<u64>(input, limits);
    } else {
        run::<u8>(input, limits);
    }
});

fn run<Q: ResourceQuantity>(input: &[u8], model_limits: Limits) {
    let mut builder = JsonValueLimits::<JsonResource, Q>::builder();
    let configured = |index: usize| -> Option<Q> {
        model_limits.maxima[index]
            .map(|maximum| Q::try_from_u64(maximum as u64).expect("header maxima fit u8 and supported quantities"))
    };
    if let Some(value) = configured(0) {
        builder = builder.max_depth(value);
    }
    if let Some(value) = configured(1) {
        builder = builder.max_nodes(value);
    }
    if let Some(value) = configured(2) {
        builder = builder.max_sequence_items(value);
    }
    if let Some(value) = configured(3) {
        builder = builder.max_map_entries(value);
    }
    if let Some(value) = configured(4) {
        builder = builder.max_key_bytes(value);
    }
    if let Some(value) = configured(5) {
        builder = builder.max_string_bytes(value);
    }
    if let Some(value) = configured(6) {
        builder = builder.max_number_bytes(value);
    }
    if let Some(value) = configured(7) {
        builder = builder.payload_bytes_limit(ResourceLimit::new(JsonResource::PayloadBytes, value));
    }

    let mut budget = builder.build().budget();
    let mut model = Model::default();
    let mut cursor = 10;
    while cursor < input.len() {
        model.begin();
        let boundary_reached = {
            let mut transaction = Some(budget.transaction());
            let mut boundary_reached = false;
            while cursor < input.len() {
                let instruction = &input[cursor..input.len().min(cursor + INSTRUCTION_SIZE)];
                cursor += INSTRUCTION_SIZE;
                let opcode = byte(instruction, 0);
                let depth = u16::from_le_bytes([byte(instruction, 1), byte(instruction, 2)]) as usize;
                let amount = u16::from_le_bytes([byte(instruction, 3), byte(instruction, 4)]) as usize;
                let boundary = usize::from(byte(instruction, 5))
                    | (usize::from(byte(instruction, 6)) << 8)
                    | (usize::from(byte(instruction, 7)) << 16);

                if opcode == 11 {
                    let expected = model.failure;
                    let result = transaction.take().expect("active transaction").commit();
                    assert_expected_result(expected, result.as_ref().err());
                    model.finish(true);
                    boundary_reached = true;
                    break;
                }
                if opcode == 12 {
                    drop(transaction.take());
                    model.finish(false);
                    boundary_reached = true;
                    break;
                }

                let operation = opcode % 13;
                if operation <= 6 {
                    let measurement = measurement(operation, depth, amount);
                    let expected = json_transaction_model::admit(&mut model, model_limits, measurement);
                    let actual = transaction.as_mut().expect("active transaction").try_admit(measurement);
                    assert_expected_result(expected, actual.as_ref().err());
                } else if operation == 7 || operation == 8 {
                    let kind = if operation == 7 {
                        JsonContainerKind::Sequence
                    } else {
                        JsonContainerKind::Map
                    };
                    let measurement = match kind {
                        JsonContainerKind::Sequence => JsonMeasurement::Array { depth, items: 0 },
                        JsonContainerKind::Map => JsonMeasurement::Object { depth, entries: 0 },
                        _ => unreachable!("container kind has no modeled measurements"),
                    };
                    let expected = json_transaction_model::admit(&mut model, model_limits, measurement);
                    let actual = transaction
                        .as_mut()
                        .expect("active transaction")
                        .try_enter_container(kind, depth);
                    assert_expected_result(expected, actual.as_ref().err());
                } else if operation == 9 || operation == 10 {
                    let kind = if operation == 9 {
                        JsonContainerKind::Sequence
                    } else {
                        JsonContainerKind::Map
                    };
                    let expected = json_transaction_model::check_count(&mut model, model_limits, kind, amount);
                    let actual = transaction
                        .as_mut()
                        .expect("active transaction")
                        .check_container_count(kind, amount);
                    assert_expected_result(expected, actual.as_ref().err());
                }

                assert_transaction(transaction.as_ref().expect("active transaction"), model, model_limits);
                if boundary & 1 != 0 {
                    let expected = model.failure;
                    let result = transaction.take().expect("active transaction").commit();
                    assert_expected_result(expected, result.as_ref().err());
                    model.finish(true);
                    boundary_reached = true;
                    break;
                }
                if boundary & 2 != 0 {
                    drop(transaction.take());
                    model.finish(false);
                    boundary_reached = true;
                    break;
                }
            }
            if !boundary_reached {
                drop(transaction.take());
                model.finish(false);
            }
            boundary_reached
        };
        assert_budget(&budget, model, model_limits);
        if !boundary_reached {
            break;
        }
    }
}

fn measurement(opcode: u8, depth: usize, amount: usize) -> JsonMeasurement {
    match opcode {
        0 => JsonMeasurement::Null { depth },
        1 => JsonMeasurement::Boolean { depth },
        2 => JsonMeasurement::String { depth, bytes: amount },
        3 => JsonMeasurement::Number { depth, bytes: amount },
        4 => JsonMeasurement::Array { depth, items: amount },
        5 => JsonMeasurement::Object { depth, entries: amount },
        _ => JsonMeasurement::Key { bytes: amount },
    }
}

fn byte(input: &[u8], index: usize) -> u8 {
    input.get(index).copied().unwrap_or_default()
}

fn assert_expected_result<Q>(expected: Option<ExpectedFailure>, actual: Option<&MeasuredBudgetError<JsonResource, Q>>)
where
    Q: ResourceQuantity,
{
    match (expected, actual) {
        (None, None) => {}
        (Some(expected), Some(actual)) => assert_failure(expected, actual),
        (expected, actual) => {
            panic!("model/API result mismatch: expected {expected:?}, actual {actual:?}")
        }
    }
}

fn assert_failure<Q>(expected: ExpectedFailure, actual: &MeasuredBudgetError<JsonResource, Q>)
where
    Q: ResourceQuantity,
{
    assert_eq!(actual.resource(), &expected.resource);
    match expected.kind {
        FailureKind::Quantity => {
            assert!(actual.budget_error().is_none());
            let quantity = actual.quantity_error().expect("quantity conversion error");
            assert_eq!(
                quantity.measurement(),
                QuantityMeasurement::Usize(expected.measured as usize)
            );
            assert_eq!(quantity.target(), std::any::type_name::<Q>());
        }
        FailureKind::Point => {
            assert!(actual.quantity_error().is_none());
            let error = actual.budget_error().expect("point budget error");
            assert_eq!(error.resource(), &expected.resource);
            assert_eq!(error.exact_observed(), quantity::<Q>(expected.measured));
            assert_eq!(error.maximum(), quantity::<Q>(expected.maximum));
        }
        FailureKind::Cumulative => {
            assert!(actual.quantity_error().is_none());
            let error = actual.budget_error().expect("cumulative budget error");
            assert_eq!(error.resource(), &expected.resource);
            assert_eq!(error.limit(), quantity::<Q>(expected.maximum));
            assert_eq!(error.remaining(), quantity::<Q>(expected.remaining));
            assert_eq!(error.requested(), quantity::<Q>(expected.measured));
        }
    }
}

fn assert_transaction<Q>(transaction: &JsonValueTransaction<'_, JsonResource, Q>, model: Model, limits: Limits)
where
    Q: ResourceQuantity,
{
    assert_eq!(
        transaction.used_nodes(),
        configured_usage::<Q>(model.working.nodes, limits.maxima[1])
    );
    assert_eq!(
        transaction.used_payload_bytes(),
        configured_usage::<Q>(model.working.payload, limits.maxima[7]),
    );
}

fn assert_budget<R, Q>(budget: &JsonValueBudget<R, Q>, model: Model, limits: Limits)
where
    R: Clone,
    Q: ResourceQuantity,
{
    assert_eq!(
        budget.used_nodes(),
        configured_usage::<Q>(model.committed.nodes, limits.maxima[1])
    );
    assert_eq!(
        budget.used_payload_bytes(),
        configured_usage::<Q>(model.committed.payload, limits.maxima[7])
    );
}

fn configured_usage<Q: ResourceQuantity>(usage: u128, maximum: Option<u128>) -> Option<Q> {
    maximum.and_then(|_| quantity::<Q>(usage))
}

fn quantity<Q: ResourceQuantity>(value: u128) -> Option<Q> {
    u64::try_from(value).ok().and_then(|value| Q::try_from_u64(value).ok())
}
