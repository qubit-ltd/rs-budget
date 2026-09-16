// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for structural resource identities.

use qubit_budget::StructureResource;

#[test]
fn test_structure_resource_is_clone_copy_and_equatable() {
    fn assert_clone_copy_and_equatable<T: Clone + Copy + PartialEq + Eq>() {}

    assert_clone_copy_and_equatable::<StructureResource>();
    assert_ne!(StructureResource::Depth, StructureResource::Nodes);
    assert_ne!(StructureResource::SequenceItems, StructureResource::MapEntries);
}
