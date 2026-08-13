// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use qubit_budget::json::JsonResource;

/// Verifies that raw and normalized input are separately measurable resources.
#[test]
fn test_json_resource_distinguishes_input_and_normalized_input() {
    assert_ne!(JsonResource::InputBytes, JsonResource::NormalizedInputBytes);
}
