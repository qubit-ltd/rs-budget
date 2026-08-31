# qubit-budget 设计文档

[English design document](design.md) · [用户手册](user_guide.zh_CN.md) · [README](../README.zh_CN.md)

本文记录 `qubit-budget` 0.4.x 的稳定记账语义，说明适配层和 Qubit 下游 crate 所依赖的契约；具体类型和调用方法仍以公共 API 文档为准。

## 分层模型

`ResourceLimit` 只检查一次观测值，不修改状态。`ResourceBudget` 消耗不可归还的有限额度，`ResourcePool` 表示需要由调用方归还的容量，`ManagedResourcePool` 则把归还绑定到 permit 的 Drop 生命周期。结构、字符串、时长和 value 辅助类型都建立在这些原语之上：单次请求被拒绝时，对应对象不会被该请求改变。

JSON 位于更高一层，并维护两套独立账本。decode/encode session 会立即保留已接受的 I/O；`JsonValueTransaction` 只暂存一个 value 的结构和 payload 测量，只有 `commit` 才会发布。

## 原子性与状态迁移

| 操作 | 立即账本 | 暂存 value 状态 | Drop 或外围操作失败后 |
| --- | --- | --- | --- |
| 接受 raw/normalized input | 保留 | 不变 | 保留 |
| 接受 writer output prefix | 保留 | 不变 | 保留 |
| 接受 value measurement | 不变 | 暂存 | 未提交时丢弃 |
| `commit` 成功 | 不变 | 发布 | 已提交用量保留 |
| value measurement 被拒绝 | 不变 | poisoned | 全部暂存用量丢弃 |

丢弃 transaction 不能撤销 callback、对象 mutation、Hasher 更新，或底层 writer 已接受的 output prefix。需要更宽业务事务的适配层必须自行选择事务边界。

## Poison 与错误优先级

第一次 value admission 失败会永久使当前 transaction 进入 poisoned 状态。后续 admission 和 `commit` 都返回该首次错误，不会发布暂存用量。I/O 失败本身不会毒化 value transaction，因为它描述的是已观察到的工作，而不是无效的暂存 value。

预算组扣减失败时，`BudgetGroupError` 会指出第一个拒绝者并保留 source error。调用方应使用 `index`、`source_error`、`remaining`、`requested` 等类型化 accessor，而不是解析 Display 文本。

## Feature 与下游边界

默认 crate 保持轻量依赖。`json`、`time`、`big-integer` 和 `big-decimal` 按需开启，其中 `big-decimal` 也会开启 `big-integer`。受 feature 控制的模块和重导出会在 docs.rs 显示所需 feature。

下游 crate 负责资源标识、具体限额、解析、I/O、等待、同步和恢复策略；不能把 value transaction 误解为整个操作自动回滚，不能复制有限 budget，也不能将错误显示文本当作稳定接口。
