# qubit-budget 用户指南

`qubit-budget` 建模有限且单调的资源消耗。使用 `ResourceLimit` 表示不可变上限，使用
`ResourceBudget` 在一次操作中保留剩余额度。失败的 `try_consume` 调用是原子的。

```rust
use qubit_budget::ResourceBudget;

let mut bytes = ResourceBudget::new("bytes", 16_u64);
bytes.try_consume(4)?;
assert_eq!(bytes.used(), 4);
# Ok::<(), qubit_budget::BudgetError<&str>>(())
```

通用嵌套数据使用 `StructureLimits` 和 `StructureBudget`；字符串与数值辅助 API 适用于其
测量语义匹配的边界。`ResourceBudget::clone` 创建独立的合法快照，可用于事务式 adapter。

## JSON 边界

需要 JSON 资源记账时启用 `json` feature。`qubit-budget` 拥有
`JsonResource`、`JsonValueLimits`、`JsonDecodeLimits`、`JsonEncodeLimits` 及其 session；
解析、规范化、遍历和 Serde adapter 仍由 `qubit-json` 提供。

```rust
use qubit_budget::json::JsonDecodeLimits;

let limits = JsonDecodeLimits::empty()
    .with_max_input_bytes(1024)
    .with_max_normalized_input_bytes(2048)
    .with_max_depth(32)
    .with_max_nodes(256);
assert_eq!(limits.max_normalized_input_bytes(), Some(2048));
```

宽松解码器会在规范化前将兼容入口
`JsonDecodeOptions::with_max_normalized_bytes` 映射到 session 的 normalized-input budget，
从而让分配准入和记账走同一条预算路径。

## 测试与限制

使用 `cargo test --all-features` 运行 JSON、可选数值和时间辅助 API 的测试。未配置的维度仍
表示为对应 session 中的 `Option`；面对不可信边界时，应显式配置原始输入、规范化输入、值和
输出上限。
