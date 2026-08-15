// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Seals the set of supported resource quantity implementations.
// qubit-style: allow source-test-pair

/// Private marker implemented only by the crate's supported unsigned integers.
pub trait ResourceQuantitySealed {}
