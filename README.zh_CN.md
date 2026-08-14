# qubit-budget

[![Rust CI](https://github.com/qubit-ltd/rs-budget/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-budget/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-budget/coverage-badge.json)](https://qubit-ltd.github.io/rs-budget/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-budget.svg?color=blue)](https://crates.io/crates/qubit-budget)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

`qubit-budget` 为 Qubit Rust 应用提供精确、有限的资源上限和记账原语。
当解析器、转换器、序列化器或 I/O 边界需要拒绝超量工作，同时避免未检查的
算术和格式相关策略时，可以使用本 crate。

## 安装

```toml
[dependencies]
qubit-budget = "0.4"
```

只有在需要对应预算类型时，才启用 `json`、`big-integer`、`big-decimal` 或
`time` feature。

## 快速开始

假设 HTTP handler 接收请求体，并且必须在 8 KiB 后停止处理。handler 可以在
数据到达时直接扣减字节，并读取剩余容量，无需维护第二个计数器：

```rust
use qubit_budget::ResourceBudget;

let mut body_budget = ResourceBudget::new("request body bytes", 8_u64);
body_budget.try_consume(3)?;
assert_eq!(body_budget.remaining(), 5);
# Ok::<(), qubit_budget::BudgetError<&str>>(())
```

`ResourceBudget::try_consume` 具有原子性：请求超过剩余容量时返回结构化错误，
预算保持不变。

## 提供的能力

| Feature | 提供内容 |
| --- | --- |
| `json` | JSON 资源标识、limits，以及可变 decode/encode session |
| `big-integer` | `BigIntegerLimits` |
| `big-decimal` | `BigDecimalLimits` |
| `time` | 基于时钟的 `TimeBudget` |

本 crate 提供 `ResourceLimit`、`ResourceBudget`、`ResourcePool`、
`StructureLimits`、`StructureBudget`，以及字符串、数值、时长和时间辅助类型。
未配置的维度使用 `Option` 表示，而不是创建一个无限预算对象。

JSON limits 和 session 保留在这里，使配置、元数据、值对象和格式适配器能够共享
同一套记账合同。JSON 解析、规范化、遍历、Serde adapter 和应用错误策略仍由
[`qubit-json`](https://crates.io/crates/qubit-json) 提供。

## 边界

本 crate 不解析 JSON、不执行 I/O、不分配输出、不选择应用上限，也不定义应用专属
错误策略。

## 延伸阅读

- [API 文档](https://docs.rs/qubit-budget)
- [English README](README.md)
- [仓库](https://github.com/qubit-ltd/rs-budget)

## 测试

```bash
# 使用默认 feature 集运行测试
cargo test

# 使用项目声明的全部 feature 运行测试
cargo test --all-features

# 运行项目 CI 检查
./ci-check.sh

# 检查代码覆盖率
./coverage.sh
```

## 许可证

Copyright (c) 2025 - 2026. Haixing Hu. All rights reserved.

本项目基于 Apache License 2.0 授权。完整许可证文本请参阅 [LICENSE](LICENSE)。

## 贡献

欢迎贡献。请遵循 Rust API 指南，及时更新公共 API 文档与测试，并在提交 Pull
Request 前运行 `./align-ci.sh` 格式化代码，运行 `./ci-check.sh` 满足 CI 要求。

## 作者

**Haixing Hu** - *Qubit Co. Ltd.*

仓库地址：[https://github.com/qubit-ltd/rs-budget](https://github.com/qubit-ltd/rs-budget)
