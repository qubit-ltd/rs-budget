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

JSON value 的记账显式且全有或全无。先暂存一个完整 value 的每项测量，只有外围
操作成功后才发布：

```rust
use qubit_budget::json::JsonMeasurement;
use qubit_budget::json::JsonValueLimits;

let mut budget = JsonValueLimits::empty()
    .with_max_nodes(8)
    .with_max_string_bytes(16)
    .budget();
let mut transaction = budget.transaction();
transaction.try_admit(JsonMeasurement::String {
    depth: 1,
    bytes: 5,
})?;
transaction.commit();
# Ok::<(), qubit_budget::MeasuredBudgetError<qubit_budget::json::JsonResource, usize>>(())
```

每次 `try_admit` 对单项测量都是原子的。`commit` 会发布暂存的全部 value nodes 和
payload bytes；丢弃 transaction（包括 panic unwind 时）只会丢弃尚未提交的 value
记账。

## JSON 原子性

`JsonDecodeAttempt` 和 `JsonEncodeAttempt` 把不可逆的 I/O 计费与可事务化的 value
记账分开。下表给出高层 JSON 集成应当保持的合同。

| 场景 | input | normalized input | value | output |
| --- | --- | --- | --- | --- |
| strict decode 成功 | 保留 | 不适用 | 提交 | 不适用 |
| strict decode 失败 | 保留 | 不适用 | 回滚 | 不适用 |
| lenient decode 失败 | 保留 | 保留 | 回滚 | 不适用 |
| 缓冲的 `Vec<u8>` output 失败 | 不适用 | 不适用 | 回滚 | 只在成功时计费；没有 `Vec` 就不计 output |
| buffered writer 部分失败 | 不适用 | 不适用 | 回滚 | 每个 accepted prefix 立即保留 |
| incremental writer 失败 | 不适用 | 不适用 | 回滚 | 每个 accepted prefix 立即保留 |
| stream 中单个 value 失败 | 跨 value 持续累计 | 跨 value 持续累计 | 仅当前 value 回滚 | 之前接受的 output 继续保留 |

attempt 一旦接受 raw input 或 normalized input，就立即计费。先得到完整 `Vec<u8>` 的
encoder 只会在完整 output 成功后计费；writer-oriented encoder 则会在 writer 接受每个
prefix 后立刻计费，之后的错误不会撤销已经被 writer 接受的字节。

transaction 是预算边界，不是通用回滚机制。丢弃 transaction 不能撤销 writer 已接受的
写入、callback 的副作用、`Hasher` 更新或对象 mutation。stream 通常为每个完整顶层
value 创建相互独立的 attempt；higher-level grouping 也可以让一个 transaction 覆盖更大
的业务操作。业务代码处理一次 rejection 后仍可继续使用该 transaction，并在最终成功时
提交其余可接纳的测量。

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
- [English user guide](doc/user_guide.md)
- [中文用户指南](doc/user_guide.zh_CN.md)
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
