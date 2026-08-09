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
# 默认 feature 集
qubit-budget = "0.4"

# 或按需启用时长预算和单调 deadline 预算
# qubit-budget = { version = "0.4", features = ["time"] }
```

## 快速开始

```rust
use qubit_budget::{ResourceBudget, ResourceLimit};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Resource {
    Nodes,
}

let mut budget = ResourceBudget::new(Resource::Nodes, ResourceLimit::new(100));
budget.try_consume(40).expect("预算足够");
assert_eq!(budget.remaining(), 60);
```

未配置的限制维度由调用方使用 `None` 表示，不创建无限或 no-op budget：

```rust
let budget: Option<ResourceBudget<Resource>> = None;
```

## 能力

- 通过 `ResourceLimit` 表示有限的不可变单维度限制；
- 通过 `ResourceBudget<R>` 使用 `u64` 进行不可归还的余额累计；
- 通过 `ResourcePool<R>` 提供失败原子、可释放并可复用的容量；
- 通过 `time` feature 提供显式 `DurationBudget<R>` 和连续
  `TimeBudget<R, C>`。

## 限制

本 crate 有意不定义 JSON、Serde、I/O、脱敏、解析器、默认限制或领域错误
策略。使用方应保留自己的公共 resource 类型，并将结构化预算错误转换为
已有错误类型。`DurationBudget` 只消费调用方显式提交的活动时长；
`TimeBudget` 会连续包含 operation、等待和 backoff 的时间流逝。

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
