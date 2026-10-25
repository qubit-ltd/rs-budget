// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Verifies that the bilingual JSON accounting documentation preserves its
//! contract.

use qubit_budget::json::JsonDecodeLimits;
use qubit_budget::json::JsonDecodeSession;
use qubit_budget::json::JsonEncodeLimits;
use qubit_budget::json::JsonEncodeSession;
use qubit_budget::json::JsonMeasurement;
use qubit_budget::json::JsonResource;
use qubit_budget::json::JsonValueLimits;

/// Required attempt outcomes that must appear as rows in the English matrix.
const ENGLISH_MATRIX_CONCEPTS: [&str; 7] = [
    "Strict decode succeeds",
    "Strict decode fails",
    "Lenient decode fails",
    "Buffered `Vec<u8>` output fails",
    "Buffered writer",
    "Incremental writer",
    "One value in a stream",
];

/// Required attempt outcomes that must appear as rows in the Chinese matrix.
const CHINESE_MATRIX_CONCEPTS: [&str; 7] = [
    "strict decode 成功",
    "strict decode 失败",
    "lenient decode 失败",
    "`Vec<u8>` output 失败",
    "writer 部分失败",
    "writer 失败",
    "stream 中单个 value",
];

/// Verifies the expected outcome categories appear in Markdown table rows,
/// without coupling the contract to row order or exact sentence wording.
fn assert_atomicity_concepts(document: &str, concepts: &[&str; 7]) {
    let table_rows = document
        .lines()
        .filter(|line| line.trim_start().starts_with('|'))
        .collect::<Vec<_>>();
    for concept in concepts {
        assert!(
            table_rows.iter().any(|row| row.contains(concept)),
            "documentation table must cover {concept}"
        );
    }
}

/// Verifies both README files show the public value-transaction example.
#[test]
fn test_readmes_document_value_transaction_example() {
    for readme in [include_str!("../../README.md"), include_str!("../../README.zh_CN.md")] {
        let measurement_import = ["use qubit_budget", "::json::JsonMeasurement;"].concat();
        let limits_import = ["use qubit_budget", "::json::JsonValueLimits;"].concat();
        assert!(readme.contains(&measurement_import));
        assert!(readme.contains(&limits_import));
        assert!(readme.contains(".max_nodes(8)"));
        assert!(readme.contains(".max_string_bytes(16)"));
        assert!(readme.contains(".build()"));
        assert!(readme.contains("let mut transaction = budget.transaction();"));
        assert!(readme.contains("JsonMeasurement::String {"));
        assert!(readme.contains("depth: 1,"));
        assert!(readme.contains("bytes: 5,"));
        assert!(readme.contains("transaction.commit()?;"));
    }
}

/// Verifies the detailed guides explain poison while README files link readers
/// to the detailed contract.
#[test]
fn test_guides_explain_poisoned_value_transactions() {
    let english_guide = include_str!("../../doc/user_guide.md");
    assert!(english_guide.contains("first failed value admission"));
    assert!(english_guide.contains("poisons") && english_guide.contains("retained error"));
    assert!(english_guide.contains("I/O failures") && english_guide.contains("do not"));
    assert!(english_guide.contains("Dropping") && english_guide.contains("staged value"));

    let chinese_guide = include_str!("../../doc/user_guide.zh_CN.md");
    assert!(chinese_guide.contains("第一次 value admission"));
    assert!(chinese_guide.contains("poisoned") && chinese_guide.contains("首次错误"));
    assert!(chinese_guide.contains("I/O 失败") && chinese_guide.contains("毒化"));
    assert!(chinese_guide.contains("丢弃") && chinese_guide.contains("暂存"));
    assert!(include_str!("../../README.md").contains("doc/design.md"));
    assert!(include_str!("../../README.zh_CN.md").contains("doc/design.zh_CN.md"));

    let rustdoc = include_str!("../../src/json/value/json_value_transaction.rs")
        .replace("///", " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(rustdoc.contains("permanently poisons the transaction"));
    assert!(rustdoc.contains("returns that error without publishing"));
    assert!(rustdoc.contains("publishing any staged state"));
}

/// Verifies the detailed user guides state the complete matrix and its critical
/// boundaries.
#[test]
fn test_all_documents_state_attempt_boundaries_and_atomicity_matrix() {
    let documents = [
        (include_str!("../../doc/user_guide.md"), &ENGLISH_MATRIX_CONCEPTS),
        (include_str!("../../doc/user_guide.zh_CN.md"), &CHINESE_MATRIX_CONCEPTS),
    ];
    for (document, concepts) in documents {
        assert_atomicity_concepts(document, concepts);
        let normalized_document = document.to_ascii_lowercase();
        assert!(document.contains("Vec"));
        assert!(document.contains("success-only") || document.contains("只在成功时计费"));
        assert!(document.contains("accepted prefix"));
        assert!(normalized_document.contains("raw input") || normalized_document.contains("raw and normalized input"));
        assert!(normalized_document.contains("normalized input"));
        assert!(document.contains("callback"));
        assert!(document.contains("Hasher"));
        assert!(normalized_document.contains("higher-level"));
        assert!(document.contains("transaction"));
        assert!(document.contains("commit"));
        assert!(normalized_document.contains("drop") || document.contains("丢弃") || document.contains("回滚"));
    }
    let english_guide = include_str!("../../doc/user_guide.md");
    let chinese_guide = include_str!("../../doc/user_guide.zh_CN.md");
    assert!(english_guide.contains("begin_value()") && english_guide.contains("explicitly"));
    assert!(chinese_guide.contains("begin_value()") && chinese_guide.contains("显式"));
}

/// Verifies the documented transaction and attempt usage against public APIs.
#[test]
fn test_documented_transaction_and_attempt_contracts_compile() {
    let mut budget = JsonValueLimits::<JsonResource, usize>::builder()
        .max_nodes(2)
        .build()
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
    transaction.commit().expect("transaction commits");
    assert_eq!(budget.used_nodes(), Some(1));

    let mut decode = JsonDecodeSession::from_limits(
        JsonDecodeLimits::<JsonResource, usize>::builder()
            .max_input_bytes(4)
            .max_normalized_input_bytes(4)
            .max_nodes(2)
            .build(),
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

    let mut encode = JsonEncodeSession::from_limits(
        JsonEncodeLimits::<JsonResource, usize>::builder()
            .max_output_bytes(4)
            .max_nodes(2)
            .build(),
    );
    {
        let mut attempt = encode.begin_value();
        attempt.try_consume_output_bytes(2).expect("accepted output fits");
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
    attempt.commit().expect("attempt commits");
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

/// Verifies every optional public module and root re-export exposes its
/// required feature in docs.rs builds.
#[test]
fn test_feature_gated_public_api_declares_docsrs_feature() {
    let lib = include_str!("../../src/lib.rs");
    assert!(lib.contains("#[cfg_attr(docsrs, doc(cfg(feature = \"json\")))]\npub mod json;"));
    assert_eq!(
        2,
        lib.matches("#[cfg_attr(docsrs, doc(cfg(feature = \"time\")))]").count()
    );
    assert_eq!(
        2,
        lib.matches("#[cfg_attr(docsrs, doc(cfg(feature = \"big-integer\")))]")
            .count()
    );
    assert_eq!(
        2,
        lib.matches("#[cfg_attr(docsrs, doc(cfg(feature = \"big-decimal\")))]")
            .count()
    );
    assert!(
        include_str!("../../src/time/mod.rs")
            .contains("#[cfg_attr(docsrs, doc(cfg(feature = \"time\")))]\npub use time_budget::TimeBudget;")
    );
    assert!(include_str!("../../src/value/mod.rs").contains(
        "#[cfg_attr(docsrs, doc(cfg(feature = \"big-integer\")))]\npub use big_integer_limits::BigIntegerLimits;"
    ));
    assert!(include_str!("../../src/value/mod.rs").contains(
        "#[cfg_attr(docsrs, doc(cfg(feature = \"big-decimal\")))]\npub use big_decimal_limits::BigDecimalLimits;"
    ));
}

/// Verifies the candidate numeric and session lifetime facts are explained in
/// both languages and link to their detailed guide.
#[test]
fn test_documents_explain_lower_bound_measurements_and_borrowed_identities() {
    for readme in [include_str!("../../README.md"), include_str!("../../README.zh_CN.md")] {
        assert!(readme.contains("AtLeastU64"));
        assert!(readme.contains("0.10"));
    }
    let english = include_str!("../../doc/user_guide.md");
    let chinese = include_str!("../../doc/user_guide.zh_CN.md");
    assert!(english.contains("proven lower bound"));
    assert!(chinese.contains("已证明的下界"));
    assert!(english.contains("borrowed from a local `String`"));
    assert!(chinese.contains("局部 `String`"));
    assert!(english.contains("from_limits") && chinese.contains("from_limits"));
    assert!(english.contains("qubit-json") && chinese.contains("qubit-json"));
    for guide in [english, chinese] {
        assert!(guide.contains("docs.rs/qubit-json"));
        assert!(guide.contains("write_incremental"));
        assert!(guide.contains("accepted output prefix") || guide.contains("output bytes"));
    }
}
