# qubit-budget

[![Rust CI](https://github.com/qubit-ltd/rs-budget/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-budget/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-budget/coverage-badge.json)](https://qubit-ltd.github.io/rs-budget/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-budget.svg?color=blue)](https://crates.io/crates/qubit-budget)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

`qubit-budget` 为 Qubit Rust crates 提供轻量、与数据格式无关的有限资源限制和记账原语。

它拥有通用的 `ResourceLimit`、`ResourceBudget`、`ResourcePool`、
`StructureLimits`、`StructureBudget`，以及字符串、数值、时长和时间辅助 API。预算始终是
有限的；未配置的维度应使用 `Option<ResourceBudget<_, _>>` 表示。

## Features

| Feature | 提供内容 |
| --- | --- |
| `json` | JSON 资源标识、limits 和 budget session |
| `big-integer` | `BigIntegerLimits` |
| `big-decimal` | `BigDecimalLimits` |
| `time` | 基于时钟的 `TimeBudget` |

最低支持 Rust 1.94。

## 快速开始

```rust
use qubit_budget::ResourceBudget;

let mut budget = ResourceBudget::new("body bytes", 8_u64);
budget.try_consume(3)?;
assert_eq!(budget.remaining(), 5);
# Ok::<(), qubit_budget::BudgetError<&str>>(())
```

`ResourceBudget` 可 clone，用于创建独立的合法记账快照；每个快照可以独立计费。

## JSON 支持

启用可选的 `json` feature 后，本 crate 拥有 JSON 资源标识、limits 和可变 session。
依赖声明为 `qubit-budget = { version = "0.4", features = ["json"] }`，并从
`qubit_budget::json` 导入 `Json*` budget 类型。解析、规范化、遍历和 Serde adapter
仍由 [`qubit-json`](https://crates.io/crates/qubit-json) 提供。

```rust
use qubit_budget::json::JsonDecodeLimits;

let limits = JsonDecodeLimits::empty()
    .with_max_input_bytes(1024)
    .with_max_nodes(128)
    .with_max_string_bytes(4096);
assert_eq!(limits.max_input_bytes(), Some(1024));
```

## 边界

本 crate 不解析 JSON、不执行 I/O、不分配输出、不选择具体上限，也不定义应用专属错误策略。

## 测试

使用 `cargo test --all-features` 运行完整 feature 集合的测试。

## 许可证

本项目采用 Apache License, Version 2.0，详见 [LICENSE](LICENSE)。

## 贡献

提交变更前请运行仓库的 `style-check.sh`、Clippy、测试和文档检查。

## 作者

Haixing Hu
