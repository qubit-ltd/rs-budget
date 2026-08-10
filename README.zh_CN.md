# rs-budget

[![Rust CI](https://github.com/qubit-ltd/rs-budget/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-budget/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-budget/coverage-badge.json)](https://qubit-ltd.github.io/rs-budget/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-budget.svg?color=blue)](https://crates.io/crates/qubit-budget)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

`qubit-budget` 为需要限制输入、遍历、输出或耗时资源的 Rust 库提供轻量的
记账原语。使用方可以自行决定资源名称、上限和错误映射，因此 transport、parser、
文件系统或脱敏 crate 能够约束有限工作量，同时保留自己的领域策略。

## 安装

```toml
[dependencies]
qubit-budget = "0.3"
```

默认 feature 集为空，按需启用扩展：

| Feature | 提供内容 |
| --- | --- |
| `json` | 用于 JSON 测量和节点记账的 `JsonLimits`、`JsonBudget` |
| `time` | `DurationBudget` 与基于时钟的 `TimeBudget` |

最低支持的 Rust 版本为 1.94。

## 快速开始

假设 HTTP 或 I/O adapter 分块接收响应体。它可以在接受每个分块前先记账，
并在边界处继续转换为自己的 response 或 stream 错误类型：

```rust
use qubit_budget::ResourceBudget;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let chunks = [b"hello".as_slice(), b" world".as_slice()];
let mut body_budget = ResourceBudget::new("response body", 11_usize);
let mut body = Vec::new();

for chunk in chunks {
    body_budget.try_consume(chunk.len())?;
    body.extend_from_slice(chunk);
}

assert_eq!(body, b"hello world");
# Ok(())
# }
```

`try_consume` 具有原子性：请求超过剩余容量时返回
`BudgetError::Insufficient`，且余额保持不变。下游 Qubit crate（如 `rs-http`、
`rs-io`、`rs-fs` 和 `rs-redact`）也使用同样的模式限制 body、stream、文件和诊断数据。

## 为什么需要这个项目

资源上限通常应由理解输入或操作的 crate 所有。如果每个 adapter 都重新实现有限
记账，就容易产生不同的失败语义和状态转换规则。本 crate 统一这些机制，同时把
资源名称、默认值、调度方式和对外错误策略留给使用方。

## 核心能力

| 需求 | 公开 API |
| --- | --- |
| 单次、包含边界的点检查 | `ResourceLimit<R, Q>` |
| 不可归还的累计消耗 | `ResourceBudget<R, Q>` |
| 支持获取和释放的容量池 | `ResourcePool<R, Q>` |
| 结构化失败信息 | `BudgetError<R, Q>`：`LimitExceeded`、`Insufficient`、`InvalidRelease` |
| 通用嵌套数据限制 | `StructureLimits`、`StructureBudget` |
| JSON 输入和遍历限制（`json`） | `JsonLimits`、`JsonBudget`、`JsonResource` |
| 显式时长或连续 deadline（`time`） | `DurationBudget<R>`、`TimeBudget<R, C>` |

数量使用精确的无符号整数，默认类型为 `u64`；结构化和 JSON 辅助类型使用 `usize`。
未配置的维度使用 `Option::None` 表示，而不是创建一个“无限”预算对象。新的预算
会话从完整配置容量开始。

## 边界与保证

- 点限制把一次观测值与包含边界在内的最大值比较，不会在多次调用之间累计。
- 资源预算单调消耗容量；失败请求不会改变状态。容量池可以显式释放容量，但不提供
  同步、等待、公平性、permit 或取消机制。
- `StructureBudget` 和 `JsonBudget` 不解析输入；由使用方 parser 或遍历逻辑决定
  测量什么，以及何时检查或记账。
- `DurationBudget` 只计算调用方显式提交的时长。`TimeBudget` 读取注入的单调时钟，
  因而会覆盖 operation、等待、排队和 backoff 的时间。
- 本 crate 不提供 I/O、Serde 集成、默认上限、重试策略、脱敏或应用专属错误类型。

## 延伸阅读

- [英文用户指南](doc/user_guide.md)
- [中文用户指南](doc/user_guide.zh_CN.md)
- [API 文档](https://docs.rs/qubit-budget)
- [English README](README.md)

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
Pull Request 前运行 `./align-ci.sh` 格式化代码，运行 `./ci-check.sh` 对齐 CI 要求。

## 作者

**Haixing Hu** - *Qubit Co. Ltd.*

仓库地址：[https://github.com/qubit-ltd/rs-budget](https://github.com/qubit-ltd/rs-budget)
