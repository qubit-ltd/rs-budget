# `qubit-budget` 用户指南

[English guide](user_guide.md) | [README](../README.zh_CN.md) | [API 文档](https://docs.rs/qubit-budget)

`qubit-budget` 将有限记账机制与领域策略分离，适合希望在解析、解码或遍历时
及时停止资源消耗、又要保留自身资源名称和对外错误模型的库作者。每个 budget
对象都表示一个已经配置的有限上限；未配置的维度使用 `Option::None`。默认数量
类型为 `u64`，结构化和 JSON 辅助类型使用 `usize`。

## 概念模型

`ResourceLimit<R, Q>` 是一个不可变的包含边界在内的单次观测上限：
`check(actual)` 成功，或返回 `BudgetError::LimitExceeded`。
`ResourceBudget<R, Q>` 记录不可归还的累计消耗，`ResourcePool<R, Q>` 支持获取
和释放可复用容量。它们统一使用 `BudgetError<R, Q>` 报错：`Insufficient` 表示
请求超出剩余容量，`InvalidRelease` 表示释放数量超过当前已使用数量。

`StructureLimits` 用于组合嵌套数据的可选限制，并创建相互独立的
`StructureBudget` 会话。启用 `json` feature 后，`JsonLimits` 和 `JsonBudget`
将同一模型用于 JSON 专属资源。通用结构类型和 JSON feature 都不负责解析数据。

## JSON 遍历场景

假设服务已经拥有 JSON parser，现在需要拒绝根深度大于 64 或遍历节点超过
100,000 的输入。启用 `json` 后，为每份输入建立一个会话；parser 观察到数据时
调用点检查，每处理一个节点时记账：

```rust
use qubit_budget::JsonLimits;

let limits = JsonLimits::new()
    .with_max_depth(64)
    .with_max_nodes(100_000);
let mut budget = limits.budget();
budget.check_depth(1)?;
budget.charge_node()?;
```

完整输入的字节数、根节点计入的深度、数组条目、对象条目、解码后的 UTF-8 字符串
字节数和数字词法表示的字节数都属于点限制。重复通过同一项点检查不会减少余额。
节点是累计限制：每次 `charge_node()` 都会消耗当前会话的一个单位，失败不会改变
会话状态。下一份输入调用 `limits.budget()` 后，会得到已恢复到配置上限的新节点
预算。

`JsonLimits` 有意不是 parser，`json` feature 也不包含 Serde 集成。识别各项
测量值并决定何时检查，仍是使用方 parser 的职责。

## 错误映射

本 crate 提供的是结构化事实，而不是领域通用错误策略。应在 parser 或 wire 的
边界，把 `BudgetError` 映射为该 crate 已经对外暴露的错误。例如，使用方可通过
`JsonResource` 和错误变体区分深度越界与累计节点耗尽。

## 其他预算类型

非 JSON 的嵌套遍历使用 `StructureLimits`。其中深度、序列条目和映射条目都是点
检查，节点与 `JsonBudget` 一样按会话累计。单维领域数量可直接使用
`ResourceLimit`、`ResourceBudget` 或 `ResourcePool`。可选 `time` feature 提供
`DurationBudget<R>`，用于调用方显式提交的活动时长；`TimeBudget<R, C>` 则提供
一个连续的单调 deadline，会包含 operation、等待、排队和 backoff 时间。

## 限制与最佳实践

本 crate 不决定字节、节点、深度或属性上限，不提供 JSON parser、Serde 集成、
I/O、脱敏、默认上限、重试策略或应用错误类型。请在拥有领域策略的边界配置上限，
为每份独立受限的输入创建新的 structure 或 JSON budget，并在那里转换
`BudgetError`。
