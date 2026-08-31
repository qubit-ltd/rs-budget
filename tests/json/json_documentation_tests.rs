// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Verifies that the bilingual JSON accounting documentation preserves its
//! contract.

use std::fs;
use std::process::Command;

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
        assert!(position >= previous, "atomicity-matrix rows must be ordered");
        previous = position;
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
    assert!(english_guide.contains("first failed value admission poisons the transaction"));
    assert!(english_guide.contains("commit") && english_guide.contains("retained error"));
    assert!(english_guide.contains("I/O failures do not"));
    assert!(english_guide.contains("Dropping") && english_guide.contains("staged value"));

    let chinese_guide = include_str!("../../doc/user_guide.zh_CN.md");
    assert!(chinese_guide.contains("第一次 value admission 失败会使 transaction 进入 poisoned 状态"));
    assert!(chinese_guide.contains("commit") && chinese_guide.contains("首次错误"));
    assert!(chinese_guide.contains("I/O 失败本身不会毒化"));
    assert!(chinese_guide.contains("丢弃") && chinese_guide.contains("暂存"));
    assert!(include_str!("../../README.md").contains("[Design document](doc/design.md)"));
    assert!(include_str!("../../README.zh_CN.md").contains("[设计文档](doc/design.zh_CN.md)"));

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
        (include_str!("../../doc/user_guide.md"), &ENGLISH_MATRIX_ROWS),
        (include_str!("../../doc/user_guide.zh_CN.md"), &CHINESE_MATRIX_ROWS),
    ];
    for (document, rows) in documents {
        assert_complete_atomicity_matrix(document, rows);
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
    assert!(
        include_str!("../../doc/user_guide.md").contains("Callers create each attempt explicitly with `begin_value()`")
    );
    assert!(
        include_str!("../../doc/user_guide.zh_CN.md").contains("调用者通过 `begin_value()` 显式创建每个 attempt。")
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

/// Extracts visible Rust snippets from a Markdown document.
fn rust_snippets(document: &str) -> Vec<String> {
    document
        .split("```rust\n")
        .skip(1)
        .filter_map(|section| section.split_once("```").map(|(snippet, _)| snippet))
        .map(|snippet| {
            snippet
                .lines()
                .filter(|line| !line.starts_with("# "))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .collect()
}

/// Verifies the actual Rust snippets in both user guides compile against the
/// current public package rather than merely resembling valid API calls.
#[test]
fn test_user_guide_rust_snippets_compile() {
    #[cfg(not(miri))]
    {
        let workspace = std::env::temp_dir().join(format!("qubit-budget-guide-snippets-{}", std::process::id()));
        let source_dir = workspace.join("src");
        fs::create_dir_all(&source_dir).expect("temporary snippet project should be created");
        let package_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        fs::write(
            workspace.join("Cargo.toml"),
            format!(
                "[package]\nname = \"qubit-budget-guide-snippets\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n[dependencies]\nqubit-budget = {{ path = {:?}, features = [\"json\"] }}\n",
                package_root
            ),
        )
        .expect("temporary snippet manifest should be written");

        let guides = [
            include_str!("../../doc/user_guide.md"),
            include_str!("../../doc/user_guide.zh_CN.md"),
        ];
        let functions = guides
            .into_iter()
            .flat_map(rust_snippets)
            .enumerate()
            .map(|(index, snippet)| {
                format!(
                    "fn guide_snippet_{index}() -> Result<(), Box<dyn std::error::Error>> {{\n{snippet}\nOk(())\n}}\n"
                )
            })
            .collect::<String>();
        fs::write(source_dir.join("main.rs"), format!("{functions}\nfn main() {{}}\n"))
            .expect("temporary snippet source should be written");

        let output = Command::new("cargo")
            .arg("+1.94.0")
            .arg("check")
            .arg("--quiet")
            .arg("--offline")
            .current_dir(&workspace)
            .env("CARGO_TARGET_DIR", workspace.join("target"))
            .output()
            .expect("Cargo should compile user-guide snippets");
        let _ = fs::remove_dir_all(&workspace);
        assert!(
            output.status.success(),
            "user-guide Rust snippets must compile:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
