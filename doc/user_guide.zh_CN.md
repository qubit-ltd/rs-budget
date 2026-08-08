# `qubit-budget` 用户指南

`qubit-budget` 将可复用的累计机制与领域策略分离。`ResourceLimit` 保存一个
不可变的单维度上限，`ResourceBudget` 保存一次操作的使用量，
`LimitExceeded<K>` 保留调用方选择的资源类别。

核心 crate 允许上限为零，并使用 `usize::MAX` 表示无限制。累计操作使用饱和
算术，因此整数溢出不能把超大请求变成合法请求。`consume` 失败时保持之前的
使用量不变；需要只检查而不修改状态时使用 `check_additional`。

本库不决定 bytes、nodes、depth 或 properties 是否需要限制，也不决定默认值
和错误文本。wire、parser、脱敏、I/O 和转换 crate 应在本地保留这些策略，
并将强类型事实转换为现有公共错误。
