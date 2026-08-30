// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Internal state shared by managed resource pools and their permits.

mod managed_resource_pool_inner;

pub(super) use managed_resource_pool_inner::ManagedResourcePoolInner;
