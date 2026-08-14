# qubit-budget 用户指南

[English user guide](user_guide.md) · [README](../README.zh_CN.md) ·
[API 文档](https://docs.rs/qubit-budget)

`qubit-budget` 用于表示有限且单调累积的资源消耗。`ResourceLimit` 负责不可变配置，
`ResourceBudget` 用于在单项资源计费后保留剩余额度；失败的 `try_consume` 不会改变该预算。

## JSON 记账模型

需要 JSON 资源记账时启用 `json` feature。`JsonValueLimits` 配置遍历和 payload 上限，
其生成的 `JsonValueBudget` 只保存已提交的用量；`JsonValueTransaction` 保存一个 value
的 working state。使用 `JsonMeasurement` 表示原生 JSON 事件，而不是分别改写节点或
payload 计数器。

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

`try_admit` 对单项测量保持原子性，拒绝后 transaction 仍然可用。`commit` 会一次发布
全部 working state；普通 drop、错误传播和 panic unwind 只会丢弃 working state，不会改变
已提交的 value budget。

## Decode 与 encode attempt

调用者通过 `begin_value()` 显式创建每个 attempt。每个 attempt 对应一个完整顶层 value，
并同时独占立即 I/O 计费和一个 value transaction，因此回滚边界清晰，而不是把整个 session
当成快照。

```rust
use qubit_budget::json::JsonDecodeLimits;
use qubit_budget::json::JsonDecodeSession;
use qubit_budget::json::JsonMeasurement;

let mut session = JsonDecodeSession::owned(
    JsonDecodeLimits::empty()
        .with_max_input_bytes(32)
        .with_max_normalized_input_bytes(32)
        .with_max_nodes(8),
);
let mut attempt = session.begin_value();
attempt.try_consume_input_bytes(5)?;
attempt.try_consume_normalized_input_bytes(5)?;
attempt.try_admit(JsonMeasurement::String { depth: 1, bytes: 5 })?;
attempt.commit();
# Ok::<(), qubit_budget::MeasuredBudgetError<qubit_budget::json::JsonResource, usize>>(())
```

encode 时使用 `JsonEncodeSession::owned` 或 `JsonEncodeSession::borrowing_output`，随后调用
`begin_value()`。`JsonEncodeAttempt::try_consume_output_bytes` 只对调用方确认已被接受的
字节计费；两类 attempt 都只通过 `commit` 发布 value accounting。

## 原子性矩阵

| 场景 | input | normalized input | value | output |
| --- | --- | --- | --- | --- |
| strict decode 成功 | 保留 | 不适用 | 提交 | 不适用 |
| strict decode 失败 | 保留 | 不适用 | 回滚 | 不适用 |
| lenient decode 失败 | 保留 | 保留 | 回滚 | 不适用 |
| 缓冲的 `Vec<u8>` output 失败 | 不适用 | 不适用 | 回滚 | 只在成功时计费；没有 `Vec` 就不计 output |
| buffered writer 部分失败 | 不适用 | 不适用 | 回滚 | 每个 accepted prefix 立即保留 |
| incremental writer 失败 | 不适用 | 不适用 | 回滚 | 每个 accepted prefix 立即保留 |
| stream 中单个 value 失败 | 跨 value 持续累计 | 跨 value 持续累计 | 仅当前 value 回滚 | 之前接受的 output 继续保留 |

raw input 与 normalized input 一经接受就立即计费并保留。先产生完整 `Vec<u8>` 的编码器可以
只在序列化成功后计 output；失败且未返回 `Vec` 时不会产生 output charge。buffered 或
incremental writer 不同：每个 accepted prefix 会立即计费；即使之后发生 budget、序列化、I/O
或 panic 错误，这些计费仍会保留。

stream 的每个 value 使用独立 attempt，后续 value 失败不能回滚先前已经提交的 value。
higher-level grouping 可以有意让一个 transaction 覆盖更大的业务操作；业务代码也可以处理
一次 rejection 后继续接纳其他测量，并在该更大操作成功时提交。

## 外部副作用与限制

value transaction 只能回滚 value accounting。drop attempt 或 transaction 不能撤销 writer
已经接受的字节、callback 的副作用、`Hasher` 更新或对象 mutation。如果业务操作要求预算
拒绝后这些外部效果仍保持不变，必须先预检再执行，或采用领域专属的恢复策略。

本 crate 提供 limits 与 accounting；解析、规范化、遍历、Serde adapter 及应用错误策略仍由
[`qubit-json`](https://crates.io/crates/qubit-json) 提供。在不可信边界，应按操作实际需要配置
raw input、normalized input、value 和 output 维度。
