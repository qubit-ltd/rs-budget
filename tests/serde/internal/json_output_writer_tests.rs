// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the bounded internal JSON output writer.

use std::io::Write;

use super::JsonOutputWriter;
use crate::JsonLimits;

/// Verifies a rejected write leaves the internal vector unchanged.
#[test]
fn test_json_output_writer_rejects_before_vec_growth() {
    let budget = JsonLimits::new().with_max_output_bytes(8).budget();
    let mut writer = JsonOutputWriter::new(&budget);

    writer
        .write(b"123456789")
        .expect_err("an oversized first write must be rejected");
    assert_eq!(writer.buffered_len(), 0);

    writer
        .write_all(b"12345678")
        .expect("the exact output limit should be accepted");
    assert_eq!(writer.buffered_len(), 8);

    writer
        .write_all(b"9")
        .expect_err("the ninth byte must be rejected");

    assert_eq!(writer.buffered_len(), 8);
}
