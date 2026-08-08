# rs-budget

[![Rust CI](https://github.com/qubit-ltd/rs-budget/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-budget/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-budget/coverage-badge.json)](https://qubit-ltd.github.io/rs-budget/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-budget.svg?color=blue)](https://crates.io/crates/qubit-budget)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

为 Qubit Rust crate 提供无领域依赖的资源限制与预算累计原语。

## 目标用户

需要限制输入、输出、深度、节点或条目处理量，同时希望把领域策略和错误
诊断保留在自身 crate 中的 Rust 库作者。

## 安装

在 `Cargo.toml` 中加入已发布的 crate：

```toml
[dependencies]
qubit-budget = "0.1"
```

## 快速开始

```rust
use qubit_budget::ResourceLimit;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResourceKind {
    Nodes,
}

let mut budget = ResourceLimit::new(100).budget();
budget.consume(ResourceKind::Nodes, 40).expect("预算足够");
assert_eq!(budget.remaining(), 60);
```

## 能力

- 通过 `ResourceLimit` 表示不可变的单维度限制；
- 通过 `ResourceBudget` 进行单次操作内的可变累计；
- 通过调用方定义的资源类别构造强类型 `LimitExceeded<K>` 事实；
- 使用饱和累计和失败原子性的消费操作。

## 限制

本 crate 有意不定义 JSON、Serde、I/O、脱敏、解析器、默认限制或领域错误
策略。使用方应保留自己的公共 limits 类型，并将 `LimitExceeded<K>` 转换为
已有错误类型。

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
