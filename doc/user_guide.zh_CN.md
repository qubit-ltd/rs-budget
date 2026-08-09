# `qubit-budget` 用户指南

`qubit-budget` 将有限的计量机制与领域策略分离。每个预算对象都表示一个已
配置的有限上限；未配置的维度统一由 `Option::None` 表示，不创建无限预算对象。
所有资源数量统一使用 `u64`。

点观察使用 `ResourceLimit`，不可归还的累计量使用 `ResourceBudget<R>`，可
获取、释放并复用的容量使用 `ResourcePool<R>`。`ResourceBudget` 保存
`remaining`，只有完整请求能够满足时才执行减法，因此失败保持原状态且没有累
计加法溢出。获取和释放统一返回 `ResourcePoolError<R>`，一个函数可以直接用
`?` 传播两种失败：

```rust
fn acquire_then_release<R: Clone>(
    pool: &mut qubit_budget::ResourcePool<R>,
    amount: u64,
) -> Result<(), qubit_budget::ResourcePoolError<R>> {
    pool.try_acquire(amount)?;
    pool.release(amount)?;
    Ok(())
}
```

启用可选 `time` feature 后，`DurationBudget<R>` 只累计调用方显式提交的活动
时长，不读取时钟；`TimeBudget<R, C>` 在注入的 `qubit-clock` 单调时钟域中固
定 deadline，operation、等待、排队和 backoff 都会自然消耗同一个端到端预算。

本库不决定 bytes、nodes、depth 或 properties 是否需要限制，也不决定默认值
和领域错误策略。wire、parser、脱敏、I/O、retry 和转换 crate 应在本地保留
这些策略，并将结构化预算事实转换为现有公共错误。
