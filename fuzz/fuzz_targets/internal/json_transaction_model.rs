// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_budget::json::JsonContainerKind;
use qubit_budget::json::JsonMeasurement;
use qubit_budget::json::JsonResource;

const DEPTH: usize = 0;
const NODES: usize = 1;
const SEQUENCE: usize = 2;
const MAP: usize = 3;
const KEY: usize = 4;
const STRING: usize = 5;
const NUMBER: usize = 6;
const PAYLOAD: usize = 7;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FailureKind {
    Quantity,
    Point,
    Cumulative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ExpectedFailure {
    pub(crate) kind: FailureKind,
    pub(crate) resource: JsonResource,
    pub(crate) measured: u128,
    pub(crate) maximum: u128,
    pub(crate) remaining: u128,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Usage {
    pub(crate) nodes: u128,
    pub(crate) payload: u128,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Model {
    pub(crate) committed: Usage,
    pub(crate) working: Usage,
    pub(crate) failure: Option<ExpectedFailure>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Limits {
    pub(crate) maxima: [Option<u128>; 8],
    pub(crate) quantity_maximum: u128,
}

impl Model {
    pub(crate) fn begin(&mut self) {
        self.working = self.committed;
        self.failure = None;
    }

    pub(crate) fn fail(&mut self, error: ExpectedFailure) -> ExpectedFailure {
        *self.failure.get_or_insert(error)
    }

    pub(crate) fn finish(&mut self, commit: bool) {
        if commit && self.failure.is_none() {
            self.committed = self.working;
        }
    }
}

pub(crate) fn admit(model: &mut Model, limits: Limits, measurement: JsonMeasurement) -> Option<ExpectedFailure> {
    if let Some(error) = model.failure {
        return Some(error);
    }

    let (depth, point_dimension, point_value, payload, node) = match measurement {
        JsonMeasurement::Null { depth } => (Some(depth), None, None, 0, true),
        JsonMeasurement::Boolean { depth } => (Some(depth), None, None, 0, true),
        JsonMeasurement::String { depth, bytes } => (Some(depth), Some(STRING), Some(bytes), bytes, true),
        JsonMeasurement::Number { depth, bytes } => (Some(depth), Some(NUMBER), Some(bytes), bytes, true),
        JsonMeasurement::Array { depth, items } => (Some(depth), Some(SEQUENCE), Some(items), 0, true),
        JsonMeasurement::Object { depth, entries } => (Some(depth), Some(MAP), Some(entries), 0, true),
        JsonMeasurement::Key { bytes } => (None, Some(KEY), Some(bytes), bytes, false),
    };

    let conversion_dimensions = [
        depth.map(|value| (DEPTH, value)),
        point_value.and_then(|value| point_dimension.map(|dimension| (dimension, value))),
    ];
    for (dimension, value) in conversion_dimensions.into_iter().flatten() {
        if limits.maxima[dimension].is_some() {
            if value as u128 > limits.quantity_maximum {
                let error = quantity_failure(resource(dimension), value as u128, limits.quantity_maximum);
                return Some(model.fail(error));
            }
        }
    }

    let payload_point =
        point_dimension.filter(|dimension| *dimension == KEY || *dimension == STRING || *dimension == NUMBER);
    let payload_conversion_dimension = payload_point
        .filter(|dimension| limits.maxima[*dimension].is_some())
        .or_else(|| limits.maxima[PAYLOAD].map(|_| PAYLOAD));
    if let Some(dimension) = payload_conversion_dimension {
        if payload as u128 > limits.quantity_maximum {
            let error = quantity_failure(resource(dimension), payload as u128, limits.quantity_maximum);
            return Some(model.fail(error));
        }
    }

    if let Some(value) = depth {
        if let Some(maximum) = limits.maxima[DEPTH] {
            if value as u128 > maximum {
                let error = point_failure(JsonResource::Depth, value as u128, maximum);
                return Some(model.fail(error));
            }
        }
    }
    if let (Some(dimension), Some(value)) = (point_dimension, point_value) {
        if let Some(maximum) = limits.maxima[dimension] {
            if value as u128 > maximum {
                let error = point_failure(resource(dimension), value as u128, maximum);
                return Some(model.fail(error));
            }
        }
    }

    if node {
        if let Some(maximum) = limits.maxima[NODES] {
            let remaining = maximum - model.working.nodes;
            if remaining < 1 {
                let error = ExpectedFailure {
                    kind: FailureKind::Cumulative,
                    resource: JsonResource::Nodes,
                    measured: 1,
                    maximum,
                    remaining,
                };
                return Some(model.fail(error));
            }
        }
    }
    if let Some(maximum) = limits.maxima[PAYLOAD] {
        let remaining = maximum - model.working.payload;
        if payload as u128 > remaining {
            let error = ExpectedFailure {
                kind: FailureKind::Cumulative,
                resource: JsonResource::PayloadBytes,
                measured: payload as u128,
                maximum,
                remaining,
            };
            return Some(model.fail(error));
        }
    }

    model.working.nodes += u128::from(node);
    model.working.payload += payload as u128;
    None
}

pub(crate) fn check_count(
    model: &mut Model,
    limits: Limits,
    kind: JsonContainerKind,
    prospective: usize,
) -> Option<ExpectedFailure> {
    if let Some(error) = model.failure {
        return Some(error);
    }
    let dimension = match kind {
        JsonContainerKind::Sequence => SEQUENCE,
        JsonContainerKind::Map => MAP,
        _ => unreachable!("container kind has no modeled dimensions"),
    };
    if let Some(maximum) = limits.maxima[dimension] {
        if prospective as u128 > limits.quantity_maximum {
            let error = quantity_failure(resource(dimension), prospective as u128, limits.quantity_maximum);
            return Some(model.fail(error));
        }
        if prospective as u128 > maximum {
            let error = point_failure(resource(dimension), prospective as u128, maximum);
            return Some(model.fail(error));
        }
    }
    None
}

fn point_failure(resource: JsonResource, measured: u128, maximum: u128) -> ExpectedFailure {
    ExpectedFailure {
        kind: FailureKind::Point,
        resource,
        measured,
        maximum,
        remaining: 0,
    }
}

fn quantity_failure(resource: JsonResource, measured: u128, maximum: u128) -> ExpectedFailure {
    ExpectedFailure {
        kind: FailureKind::Quantity,
        resource,
        measured,
        maximum,
        remaining: 0,
    }
}

fn resource(dimension: usize) -> JsonResource {
    match dimension {
        DEPTH => JsonResource::Depth,
        NODES => JsonResource::Nodes,
        SEQUENCE => JsonResource::SequenceItems,
        MAP => JsonResource::MapEntries,
        KEY => JsonResource::KeyBytes,
        STRING => JsonResource::StringBytes,
        NUMBER => JsonResource::NumberBytes,
        PAYLOAD => JsonResource::PayloadBytes,
        _ => unreachable!("dimension index is fixed by the model"),
    }
}
