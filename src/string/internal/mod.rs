// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Private implementation details for budgeted string rendering.

mod fmt_writer;
mod io_writer;
mod writer_failure;

pub(super) use fmt_writer::FmtWriter;
pub(super) use io_writer::IoWriter;
pub(super) use writer_failure::WriterFailure;
