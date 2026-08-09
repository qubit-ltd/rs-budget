# `qubit-budget` 用户指南

`qubit-budget` 将可复用的计量机制与领域策略分离。`ResourceLimit` 保存一个
不可变的单维度上限，`ResourceBudget` 保存一次操作的剩余额度，
`LimitExceeded<K>` 保留调用方选择的资源类别。

核心 crate 允许上限为零，并使用 `usize::MAX` 表示无限制。
`try_consume` 在额度不足时保持状态不变；`consume_or_exhaust` 在返回错误前
将剩余额度清零；`consume_available` 尽可能消费；`release` 归还之前消费的
额度。`ResourceBudget` 不实现 `Clone` 或 `Copy`，避免重复复制同一计量状态。

本库不决定 bytes、nodes、depth 或 properties 是否需要限制，也不决定默认值
和错误文本。wire、parser、脱敏、I/O 和转换 crate 应在本地保留这些策略，
并将强类型事实转换为现有公共错误。
