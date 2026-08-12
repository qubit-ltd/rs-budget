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

JSON 专用的 limits、session、遍历、解码、编码和 Serde 错误由 `qubit-json` 提供。因此 JSON
边界应从 `qubit_budget` 导入通用限制，并从 `qubit_json` 导入所有 JSON API。
