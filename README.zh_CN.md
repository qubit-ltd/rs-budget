# qubit-budget

`qubit-budget` 为 Qubit Rust crates 提供轻量、与数据格式无关的有限资源限制和记账原语。

它拥有通用的 `ResourceLimit`、`ResourceBudget`、`ResourcePool`、
`StructureLimits`、`StructureBudget`，以及字符串、数值、时长和时间辅助 API。预算始终是
有限的；未配置的维度应使用 `Option<ResourceBudget<_, _>>` 表示。

## Features

| Feature | 提供内容 |
| --- | --- |
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

JSON 遍历、解析、Serde adapter、session 和 JSON 资源限制属于
[`qubit-json`](https://crates.io/crates/qubit-json)。通用资源和结构限制继续从
`qubit-budget` 使用；所有 `Json*` 类型以及 JSON 编解码 API 都应从 `qubit_json` 导入。

## 边界

本 crate 不解析 JSON、不执行 I/O、不分配输出、不选择具体上限，也不定义应用专属错误策略。
