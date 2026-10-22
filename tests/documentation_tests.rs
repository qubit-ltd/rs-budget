// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Compiles, runs, and checks links in the bilingual project documentation.

#[cfg(not(miri))]
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

const DOCUMENTS: [&str; 6] = [
    "README.md",
    "README.zh_CN.md",
    "doc/user_guide.md",
    "doc/user_guide.zh_CN.md",
    "doc/design.md",
    "doc/design.zh_CN.md",
];

#[cfg(not(miri))]
#[derive(Clone)]
struct Snippet {
    document: String,
    index: usize,
    features: Vec<String>,
    source: String,
}

#[cfg(not(miri))]
fn document_snippets(document: &str, name: &str) -> Vec<(usize, String)> {
    let mut snippets = Vec::new();
    let mut current: Option<(usize, String, String)> = None;
    let mut next_index = 0;

    for (line_index, line) in document.lines().enumerate() {
        let trimmed = line.trim();
        if let Some((index, fence, source)) = &mut current {
            if trimmed == fence {
                assert!(
                    source.contains("fn main("),
                    "{name}: Rust block {index} must contain its own main function"
                );
                assert!(
                    !source
                        .lines()
                        .any(|source_line| source_line.trim_start().starts_with("# ")),
                    "{name}: Rust block {index} must not use hidden Rustdoc lines"
                );
                snippets.push((*index, source.clone()));
                current = None;
            } else {
                source.push_str(line);
                source.push('\n');
            }
            continue;
        }

        if let Some(info) = trimmed.strip_prefix("```") {
            let info = info.trim();
            if info == "rust" {
                current = Some((next_index, "```".to_owned(), String::new()));
                next_index += 1;
            } else if info.starts_with("rust") {
                panic!("{name}: unsupported Rust fence mode at line {}: {info}", line_index + 1);
            }
        }
    }

    assert!(current.is_none(), "{name}: unterminated Rust code fence");
    snippets
}

#[cfg(not(miri))]
fn parse_manifest(root: &Path) -> Vec<Snippet> {
    let manifest = fs::read_to_string(root.join("tests/fixtures/documentation-snippets.tsv"))
        .expect("read documentation snippet manifest");
    let mut seen = BTreeSet::new();
    let mut snippets = Vec::new();

    for (line_index, line) in manifest.lines().enumerate() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        assert_eq!(fields.len(), 4, "fixture line {} must have four fields", line_index + 1);
        let document = fields[0];
        assert!(DOCUMENTS.contains(&document), "unknown document {document}");
        let index = fields[1]
            .parse::<usize>()
            .expect("snippet index must be a non-negative integer");
        let features = if fields[2] == "-" {
            Vec::new()
        } else {
            fields[2].split(',').map(str::to_owned).collect()
        };
        assert!(
            features.iter().all(|feature| matches!(feature.as_str(), "json")),
            "unsupported documentation feature list on line {}",
            line_index + 1
        );
        assert_eq!(fields[3], "run", "only executable snippets are supported");
        assert!(seen.insert((document.to_owned(), index)), "duplicate fixture row");
        let source = fs::read_to_string(root.join(document)).expect("read listed document");
        let extracted = document_snippets(&source, document);
        let snippet = extracted
            .get(index)
            .unwrap_or_else(|| panic!("{document}: missing Rust block {index}"));
        snippets.push(Snippet {
            document: document.to_owned(),
            index,
            features,
            source: snippet.1.clone(),
        });
    }

    for document in DOCUMENTS {
        let source = fs::read_to_string(root.join(document)).expect("read documented file");
        let extracted = document_snippets(&source, document);
        let listed = seen
            .iter()
            .filter_map(|(listed_document, index)| (listed_document == document).then_some(*index))
            .collect::<BTreeSet<_>>();
        let actual = (0..extracted.len()).collect::<BTreeSet<_>>();
        assert_eq!(
            listed, actual,
            "{document}: snippet list must match Rust blocks exactly"
        );
    }
    snippets
}

fn heading_slug(heading: &str) -> String {
    let mut slug = String::new();
    let mut dash_pending = false;
    for character in heading.trim().trim_start_matches('#').trim().to_lowercase().chars() {
        if character.is_alphanumeric() || character == '_' || character == '-' {
            if dash_pending && !slug.is_empty() && !slug.ends_with('-') {
                slug.push('-');
            }
            slug.push(character);
            dash_pending = false;
        } else if character.is_whitespace() {
            dash_pending = true;
        }
    }
    slug
}

fn check_relative_links(root: &Path, document_name: &str) {
    let document_path = root.join(document_name);
    let document = fs::read_to_string(&document_path).expect("read document for link check");
    let mut in_fence = false;
    let mut headings = BTreeSet::new();
    for line in document.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if !in_fence && trimmed.starts_with('#') {
            headings.insert(heading_slug(trimmed));
        }
    }

    let link_pattern = regexless_links(&document);
    for (line_number, target) in link_pattern {
        if target.contains("://") || target.starts_with("mailto:") || target.starts_with('#') {
            if let Some(fragment) = target.strip_prefix('#') {
                assert!(
                    headings.contains(&heading_slug(fragment)),
                    "{document_name}:{line_number}: missing anchor #{fragment}"
                );
            }
            continue;
        }
        let (path_part, fragment) = target.split_once('#').unwrap_or((&target, ""));
        if path_part.is_empty() {
            if !fragment.is_empty() {
                assert!(
                    headings.contains(&heading_slug(fragment)),
                    "{document_name}:{line_number}: missing anchor #{fragment}"
                );
            }
            continue;
        }
        let linked = document_path.parent().unwrap().join(path_part.replace("%20", " "));
        assert!(
            linked.exists(),
            "{document_name}:{line_number}: missing local target {path_part}"
        );
        if !fragment.is_empty() && linked.is_file() && linked.extension().is_some_and(|extension| extension == "md") {
            let linked_document = fs::read_to_string(&linked).expect("read linked markdown target");
            let linked_headings = linked_document
                .lines()
                .filter(|line| line.trim().starts_with('#'))
                .map(|line| heading_slug(line.trim()))
                .collect::<BTreeSet<_>>();
            assert!(
                linked_headings.contains(&heading_slug(fragment)),
                "{document_name}:{line_number}: missing anchor {path_part}#{fragment}"
            );
        }
    }
}

fn regexless_links(document: &str) -> Vec<(usize, String)> {
    let mut links = Vec::new();
    let mut in_fence = false;
    for (line_index, line) in document.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        let mut rest = line;
        while let Some(open) = rest.find("](") {
            let after = &rest[open + 2..];
            if let Some(close) = after.find(')') {
                links.push((line_index + 1, after[..close].trim_matches(['<', '>']).to_owned()));
                rest = &after[close + 1..];
            } else {
                break;
            }
        }
    }
    links
}

#[cfg(not(miri))]
fn run_group(root: &Path, features: &[String], snippets: &[Snippet]) {
    let fixture = tempfile::tempdir().expect("create temporary documentation fixture");
    let bins = fixture.path().join("src/bin");
    fs::create_dir_all(&bins).expect("create snippet source directory");
    let feature_text = features
        .iter()
        .map(|feature| format!("{feature:?}"))
        .collect::<Vec<_>>()
        .join(",");
    let manifest = format!(
        "[package]\nname=\"budget-doc-fixture\"\nversion=\"0.0.0\"\nedition=\"2024\"\n[workspace]\n[dependencies]\nqubit-budget={{path={:?},default-features=false,features=[{}]}}\n",
        root, feature_text,
    );
    fs::write(fixture.path().join("Cargo.toml"), manifest).expect("write fixture manifest");
    fs::copy(
        root.join("rust-toolchain.toml"),
        fixture.path().join("rust-toolchain.toml"),
    )
    .expect("copy pinned toolchain into fixture");

    let mut binaries = Vec::new();
    for (serial, snippet) in snippets.iter().enumerate() {
        assert!(
            snippet.source.contains("fn main("),
            "{} Rust block {} must be a full program",
            snippet.document,
            snippet.index
        );
        assert!(
            !snippet.source.lines().any(|line| line.trim_start().starts_with("# ")),
            "{} Rust block {} must not contain hidden lines",
            snippet.document,
            snippet.index
        );
        let binary = format!("snippet_{serial}");
        fs::write(bins.join(format!("{binary}.rs")), &snippet.source).expect("write original documentation snippet");
        binaries.push(binary);
    }

    let target = fixture.path().join("target");
    let output = std::process::Command::new("cargo")
        .args(["build", "--offline", "--bins"])
        .current_dir(fixture.path())
        .env("CARGO_TARGET_DIR", &target)
        .output()
        .expect("run Cargo for documentation fixture");
    assert!(
        output.status.success(),
        "documentation snippets failed to compile (group: {}):\n{}",
        snippets
            .iter()
            .map(|snippet| format!("{}#{}", snippet.document, snippet.index))
            .collect::<Vec<_>>()
            .join(", "),
        String::from_utf8_lossy(&output.stderr)
    );

    for (binary, snippet) in binaries.iter().zip(snippets) {
        let executable = target
            .join("debug")
            .join(format!("{binary}{}", std::env::consts::EXE_SUFFIX));
        let result = std::process::Command::new(executable)
            .output()
            .expect("execute documentation snippet");
        assert!(
            result.status.success(),
            "{} Rust block {} failed:\n{}",
            snippet.document,
            snippet.index,
            String::from_utf8_lossy(&result.stderr)
        );
    }
}

#[test]
fn test_documentation_local_links_resolve() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for document in DOCUMENTS {
        check_relative_links(&root, document);
    }
}

#[cfg(not(miri))]
#[test]
fn test_documentation_rust_blocks_build_and_run_verbatim() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let snippets = parse_manifest(&root);
    let mut groups: BTreeMap<Vec<String>, Vec<Snippet>> = BTreeMap::new();
    for snippet in snippets {
        groups.entry(snippet.features.clone()).or_default().push(snippet);
    }
    for (features, snippets) in groups {
        run_group(&root, &features, &snippets);
    }
}

#[cfg(not(miri))]
#[test]
fn test_documentation_runner_rejects_compile_and_runtime_mutations() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let guide =
        fs::read_to_string(root.join("doc/user_guide.md")).expect("read English guide for disposable mutations");
    let duration = document_snippets(&guide, "doc/user_guide.md")
        .into_iter()
        .find(|(index, _)| *index == 4)
        .expect("duration program is listed at index 4")
        .1;

    let compile_mutation = duration.replace(
        "Result<(), Box<dyn std::error::Error>>",
        "Result<(), qubit_budget::InsufficientBudgetError<&'static str, u32>>",
    );
    assert_ne!(compile_mutation, duration, "compile mutation must change the program");
    let compile_result = std::panic::catch_unwind(|| {
        run_group(
            &root,
            &[],
            &[Snippet {
                document: "doc/user_guide.md".to_owned(),
                index: 4,
                features: Vec::new(),
                source: compile_mutation,
            }],
        );
    });
    assert!(compile_result.is_err(), "runner must reject a bad concrete error type");

    let runtime_mutation = duration.replace("assert_eq!(attempts.used(), 1);", "assert_eq!(attempts.used(), 99);");
    assert_ne!(runtime_mutation, duration, "runtime mutation must change the assertion");
    let runtime_result = std::panic::catch_unwind(|| {
        run_group(
            &root,
            &[],
            &[Snippet {
                document: "doc/user_guide.md".to_owned(),
                index: 4,
                features: Vec::new(),
                source: runtime_mutation,
            }],
        );
    });
    assert!(runtime_result.is_err(), "runner must reject a failing assertion");
}
