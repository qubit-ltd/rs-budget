// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================
//! Tests for lexical JSON preflight behavior and stack safety.

use std::process::Command;

use qubit_budget::BudgetError;
use qubit_budget::JsonLimits;
use qubit_budget::JsonResource;
use qubit_budget::JsonSerdeError;
use qubit_budget::from_slice_with_budget;
use qubit_budget::to_vec_with_budget;
use serde_json::Value;
use serde_json::value::RawValue;

/// Environment marker used by the stack-safety subprocess.
const DEEP_PREFLIGHT_CHILD: &str = "QUBIT_BUDGET_DEEP_PREFLIGHT_CHILD";

/// Verifies that the preflight walker charges the root and child nodes.
#[test]
fn test_json_preflight_charges_root_and_child_nodes() {
    let mut budget = JsonLimits::new().with_max_nodes(1).budget();
    let error =
        from_slice_with_budget::<Value, _>(br#"{"value":true}"#, &mut budget)
            .expect_err("the child node should exceed the node budget");

    assert!(matches!(
        error,
        JsonSerdeError::Budget(BudgetError::Insufficient {
            resource: JsonResource::Nodes,
            limit: 1,
            remaining: 0,
            requested: 1,
        })
    ));
}

/// Verifies lexical preflight handles deeply nested valid JSON without using
/// the native call stack for document depth.
#[test]
fn test_json_preflight_handles_deep_json_without_stack_overflow() {
    if std::env::var_os(DEEP_PREFLIGHT_CHILD).is_some() {
        let depth = 100_000_usize;
        let mut input = String::with_capacity(depth.saturating_mul(2).saturating_add(4));
        input.extend(std::iter::repeat_n('[', depth));
        input.push_str("null");
        input.extend(std::iter::repeat_n(']', depth));
        let mut budget = JsonLimits::new().budget();
        from_slice_with_budget::<serde::de::IgnoredAny, _>(input.as_bytes(), &mut budget)
            .expect("deep valid JSON must complete lexical preflight");

        let raw = RawValue::from_string(input)
            .expect("deep valid JSON must construct a raw-value fixture");
        let mut budget = JsonLimits::new().budget();
        to_vec_with_budget(&raw, &mut budget)
            .expect("deep RawValue and final output preflight must remain stack-safe");
        return;
    }

    let executable = std::env::current_exe().expect("test executable path must be available");
    let output = Command::new(executable)
        .arg("test_json_preflight_handles_deep_json_without_stack_overflow")
        .arg("--nocapture")
        .env(DEEP_PREFLIGHT_CHILD, "1")
        .output()
        .expect("deep preflight subprocess must start");

    assert!(
        output.status.success(),
        "deep preflight subprocess failed: status={:?}, stdout={}, stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}
