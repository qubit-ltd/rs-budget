# rs-budget

[![Rust CI](https://github.com/qubit-ltd/rs-budget/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-budget/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-budget/coverage-badge.json)](https://qubit-ltd.github.io/rs-budget/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-budget.svg?color=blue)](https://crates.io/crates/qubit-budget)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

为 Qubit Rust crate 提供轻量的有限资源限制和预算记账原语。使用方可以约束
自身的输入与处理过程，同时继续掌握领域策略和对外错误类型。

## 目标用户

适合需要限制输入、深度、节点、条目或字节处理量的 Rust 库作者；资源分类、
默认值、诊断信息和错误模型仍由使用方定义。

## 安装

在 `Cargo.toml` 中加入已发布的 crate：

```toml
[dependencies]
# 默认 feature 集
qubit-budget = "0.3"

# 仅在需要 JSON 限制记账时启用 json feature
# qubit-budget = { version = "0.3", features = ["json"] }

# 按需启用时长预算和单调 deadline 预算
# qubit-budget = { version = "0.3", features = ["time"] }
```

## 快速开始

已有 JSON 遍历逻辑的解码器可以自行配置上限并在遍历时记账。`json` feature
不提供 Serde 集成，也不提供解析器；它只提供限制配置与记账 API。

```rust
use qubit_budget::JsonLimits;

let limits = JsonLimits::new()
    .with_max_depth(64)
    .with_max_nodes(100_000);
let mut budget = limits.budget();
budget.check_depth(1)?;
budget.charge_node()?;
```

`check_depth`、输入字节、数组条目、对象条目、字符串字节和数字字节都是点检查：
每次都把一个观测值与包含边界在内的上限独立比较。`charge_node` 则会消耗当前
会话的累计节点预算；再次调用 `limits.budget()` 会创建一个拥有全新节点余额的
会话。

## 能力

- `ResourceLimit<R, Q>` 用于检查一个包含边界在内的点值。
- `ResourceBudget<R, Q>` 用于有限、不可归还的累计消耗；
  `ResourcePool<R, Q>` 用于有限且可释放的容量。
- `BudgetError<R, Q>` 是统一的结构化失败类型：点检查失败为
  `LimitExceeded`，预算消耗或获取容量失败为 `Insufficient`，超量释放为
  `InvalidRelease`。
- `StructureLimits` 生成 `StructureBudget` 会话，可限制深度、累计节点、序列
  条目和映射条目，但不绑定任何数据格式。
- 可选 `json` feature 提供 `JsonLimits` 和 `JsonBudget`，覆盖 JSON 输入字节、
  即完整输入的字节数、根节点计入的深度、累计节点、数组/对象大小、解码后的
  UTF-8 字符串字节数以及数字词法表示的字节数。
- 可选 `time` feature 提供显式 `DurationBudget<R>` 与连续的
  `TimeBudget<R, C>`。

## 错误边界

`BudgetError` 只描述限制失败的事实，并不决定应用策略。使用方应在自己的边界
根据资源和值的变体匹配它，再转换为本领域的公开错误。点限制、累计预算与容量池
失败都由同一个错误类型承载。

## 限制

本 crate 有意不提供 JSON 解析器、Serde 集成、I/O、脱敏、默认上限、重试策略
或领域错误策略。parser 或 wire crate 决定何时执行检查与节点记账，并将
`BudgetError` 转换为已有错误模型。`DurationBudget` 只消费调用方显式提交的
时长；`TimeBudget` 会包含 operation、等待、排队和 backoff 的时间流逝。

## 延伸阅读

可阅读[英文用户指南](doc/user_guide.md)、[中文用户指南](doc/user_guide.zh_CN.md)
或 [API 文档](https://docs.rs/qubit-budget)。

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

本项目基于 Apache License 2.0 授权。完整许可证文本请参阅
[LICENSE](LICENSE)。

## 贡献

欢迎贡献。请遵循 Rust API 指南，及时更新公共 API 文档与测试，并在提交
Pull Request 前运行 `./align-ci.sh`格式化代码，运行`./ci-check.sh`对齐CI要求。

## 作者

**Haixing Hu** - *Qubit Co. Ltd.*

仓库地址：[https://github.com/qubit-ltd/rs-budget](https://github.com/qubit-ltd/rs-budget)
