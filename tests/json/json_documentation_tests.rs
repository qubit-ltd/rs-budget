// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Verifies that the bilingual JSON accounting documentation preserves its
//! contract.

/// English matrix rows in their required order.
const ENGLISH_MATRIX_ROWS: [&str; 7] = [
    "| Strict decode succeeds |",
    "| Strict decode fails |",
    "| Lenient decode fails |",
    "| Buffered `Vec<u8>` output fails |",
    "| Buffered writer partially fails |",
    "| Incremental writer fails |",
    "| One value in a stream fails |",
];

/// Simplified Chinese matrix rows in their required order.
const CHINESE_MATRIX_ROWS: [&str; 7] = [
    "| strict decode 成功 |",
    "| strict decode 失败 |",
    "| lenient decode 失败 |",
    "| 缓冲的 `Vec<u8>` output 失败 |",
    "| buffered writer 部分失败 |",
    "| incremental writer 失败 |",
    "| stream 中单个 value 失败 |",
];

/// Verifies a matrix contains all seven rows in their documented order.
fn assert_complete_atomicity_matrix(document: &str, rows: &[&str; 7]) {
    let mut previous = 0;
    for row in rows {
        let position = document
            .find(row)
            .expect("documentation must contain every atomicity-matrix row");
        assert!(
            position >= previous,
            "atomicity-matrix rows must be ordered"
        );
        previous = position;
    }
}

/// Verifies both README files show the public value-transaction example.
#[test]
fn test_readmes_document_value_transaction_example() {
    for readme in [
        include_str!("../../README.md"),
        include_str!("../../README.zh_CN.md"),
    ] {
        let measurement_import = ["use qubit_budget", "::json::JsonMeasurement;"].concat();
        let limits_import = ["use qubit_budget", "::json::JsonValueLimits;"].concat();
        assert!(readme.contains(&measurement_import));
        assert!(readme.contains(&limits_import));
        assert!(readme.contains(".with_max_nodes(8)"));
        assert!(readme.contains(".with_max_string_bytes(16)"));
        assert!(readme.contains("let mut transaction = budget.transaction();"));
        assert!(readme.contains("JsonMeasurement::String {"));
        assert!(readme.contains("depth: 1,"));
        assert!(readme.contains("bytes: 5,"));
        assert!(readme.contains("transaction.commit();"));
    }
}

/// Verifies all documents state the complete matrix and its critical
/// boundaries.
#[test]
fn test_all_documents_state_attempt_boundaries_and_atomicity_matrix() {
    let documents = [
        (include_str!("../../README.md"), &ENGLISH_MATRIX_ROWS),
        (include_str!("../../README.zh_CN.md"), &CHINESE_MATRIX_ROWS),
        (
            include_str!("../../doc/user_guide.md"),
            &ENGLISH_MATRIX_ROWS,
        ),
        (
            include_str!("../../doc/user_guide.zh_CN.md"),
            &CHINESE_MATRIX_ROWS,
        ),
    ];
    for (document, rows) in documents {
        assert_complete_atomicity_matrix(document, rows);
        let normalized_document = document.to_ascii_lowercase();
        assert!(document.contains("Vec"));
        assert!(document.contains("success-only") || document.contains("只在成功时计费"));
        assert!(document.contains("accepted prefix"));
        assert!(
            normalized_document.contains("raw input")
                || normalized_document.contains("raw and normalized input")
        );
        assert!(normalized_document.contains("normalized input"));
        assert!(document.contains("callback"));
        assert!(document.contains("Hasher"));
        assert!(normalized_document.contains("higher-level"));
        assert!(document.contains("transaction"));
        assert!(document.contains("commit"));
        assert!(
            normalized_document.contains("drop")
                || document.contains("丢弃")
                || document.contains("回滚")
        );
    }
    assert!(
        include_str!("../../doc/user_guide.md")
            .contains("Callers create each attempt explicitly with `begin_value()`")
    );
    assert!(
        include_str!("../../doc/user_guide.zh_CN.md")
            .contains("调用者通过 `begin_value()` 显式创建每个 attempt。")
    );
}

/// Verifies the documented transaction and attempt usage against public APIs.
#[test]
fn test_documented_transaction_and_attempt_contracts_compile() {
    use qubit_budget::json::JsonDecodeLimits;
    use qubit_budget::json::JsonDecodeSession;
    use qubit_budget::json::JsonEncodeLimits;
    use qubit_budget::json::JsonEncodeSession;
    use qubit_budget::json::JsonMeasurement;
    use qubit_budget::json::JsonResource;
    use qubit_budget::json::JsonValueLimits;

    let mut budget = JsonValueLimits::<JsonResource, usize>::new()
        .with_max_nodes(2)
        .budget();
    {
        let mut transaction = budget.transaction();
        transaction
            .try_admit(JsonMeasurement::Null { depth: 1 })
            .expect("staged value fits");
    }
    assert_eq!(budget.used_nodes(), Some(0));
    let mut transaction = budget.transaction();
    transaction
        .try_admit(JsonMeasurement::Null { depth: 1 })
        .expect("committed value fits");
    transaction.commit();
    assert_eq!(budget.used_nodes(), Some(1));

    let mut decode = JsonDecodeSession::owned(
        JsonDecodeLimits::<JsonResource, usize>::new()
            .with_max_input_bytes(4)
            .with_max_normalized_input_bytes(4)
            .with_max_nodes(2),
    );
    {
        let mut attempt = decode.begin_value();
        attempt.try_consume_input_bytes(2).expect("input fits");
        attempt
            .try_consume_normalized_input_bytes(2)
            .expect("normalized input fits");
        attempt
            .try_admit(JsonMeasurement::Null { depth: 1 })
            .expect("staged value fits");
    }
    assert_eq!(decode.input_budget().expect("input budget").used(), 2);
    assert_eq!(
        decode
            .normalized_input_budget()
            .expect("normalized input budget")
            .used(),
        2
    );
    assert_eq!(decode.value_budget().used_nodes(), Some(0));

    let mut encode = JsonEncodeSession::owned(
        JsonEncodeLimits::<JsonResource, usize>::new()
            .with_max_output_bytes(4)
            .with_max_nodes(2),
    );
    {
        let mut attempt = encode.begin_value();
        attempt
            .try_consume_output_bytes(2)
            .expect("accepted output fits");
        attempt
            .try_admit(JsonMeasurement::Null { depth: 1 })
            .expect("staged value fits");
    }
    assert_eq!(encode.output_budget().expect("output budget").used(), 2);
    assert_eq!(encode.value_budget().used_nodes(), Some(0));
    let mut attempt = encode.begin_value();
    attempt
        .try_admit(JsonMeasurement::Null { depth: 1 })
        .expect("committed value fits");
    attempt.commit();
    assert_eq!(encode.value_budget().used_nodes(), Some(1));
}

/// Verifies the documentation no longer directs callers to removed JSON
/// writers.
#[test]
fn test_json_documentation_omits_removed_mutation_apis() {
    for document in [
        include_str!("../../README.md"),
        include_str!("../../README.zh_CN.md"),
        include_str!("../../doc/user_guide.md"),
        include_str!("../../doc/user_guide.zh_CN.md"),
    ] {
        assert!(!document.contains("enter_node"));
        assert!(!document.contains("consume_string_bytes"));
        assert!(!document.contains("value_budget_mut"));
    }
}
