# qubit-budget 用户手册

[English user guide](user_guide.md) · [中文 README](../README.zh_CN.md) ·
[API 文档](https://docs.rs/qubit-budget)

本手册适用于 `qubit-budget` 0.4.x 和 Rust 1.94 或更高版本。它面向需要约束不可信
输入或高成本操作的库作者、服务开发者；crate 不绑定具体解析器、序列化器或运行时。

## 先分清：上限、预算还是资源池？

每项约束由资源名称 `R` 和精确的无符号数量 `Q` 组成。资源名称会出现在错误中；
`Q` 可为 `u8`、`u16`、`u32`、`u64`、`u128` 或 `usize`。某个维度为 `None` 只表示
没有为它配置限制，并不代表 crate 创建了隐藏的“无限预算”。

| 你的需求 | 使用的类型 | 关键规则 |
| --- | --- | --- |
| 校验一次测量 | `ResourceLimit` | 从不修改状态。 |
| 消耗一份有限额度 | `ResourceBudget` | `try_*` 失败时预算不变。 |
| 使用后归还容量 | `ResourcePool` | 集成层必须显式归还。 |

`ResourceBudget` 不实现 `Clone`，以免有限额度被复制。`ResourcePool` 不是信号量：
它没有锁、等待、公平性、所有权跟踪或 RAII permit。

## 贯穿场景：接收一份不可信 JSON 文档

假设网关只接受满足以下条件的一份 JSON 文档：

- 原始输入和规范化输入各不超过 64 字节；
- 根节点算第 1 层，最大深度为 3；
- 总节点数不超过 8；
- 单个字符串不超过 16 个 UTF-8 字节，累计 payload 不超过 32 字节；
- 只有完整文档成功后，JSON value 用量才正式入账。

`qubit-budget` 不会替你解析 JSON。解析器或适配层应在接受输入时报告字节数，并在
遍历每个 value 或对象键时发出一个 `JsonMeasurement`。

## 安装与配置

核心类型不需要 feature：

```toml
[dependencies]
qubit-budget = "0.4"
```

本场景需要开启 `json`：

```toml
[dependencies]
qubit-budget = { version = "0.4", features = ["json"] }
```

其他可选 feature 是 `big-integer`、`big-decimal`（会启用 `big-integer`）和 `time`。

## 核心流程

### 1. 为自己负责的边界创建 session

一次独立的解码操作可使用自持有 session：

```rust
use qubit_budget::json::JsonDecodeLimits;
use qubit_budget::json::JsonDecodeSession;
use qubit_budget::json::JsonMeasurement;
use qubit_budget::json::JsonResource;

let mut session = JsonDecodeSession::owned(
    JsonDecodeLimits::<JsonResource, usize>::new()
        .with_max_input_bytes(64)
        .with_max_normalized_input_bytes(64)
        .with_max_depth(3)
        .with_max_nodes(8)
        .with_max_string_bytes(16)
        .with_max_payload_bytes(32),
);

let mut attempt = session.begin_value();
attempt.try_consume_input_bytes(7)?;
attempt.try_consume_normalized_input_bytes(7)?;
attempt.try_admit(JsonMeasurement::String {
    depth: 1,
    bytes: 5,
})?;

assert_eq!(attempt.used_nodes(), Some(1));
attempt.commit();
assert_eq!(session.input_budget().expect("configured input").used(), 7);
assert_eq!(session.value_budget().used_nodes(), Some(1));
# Ok::<(), qubit_budget::MeasuredBudgetError<qubit_budget::json::JsonResource, usize>>(())
```

`new()` 不会预设任何限制；只添加当前边界真正要执行的维度。深度包含根节点；字符串
和对象键按 UTF-8 字节数计算；payload 由对象键、字符串和数字的字节数累计而成。

### 2. 把解析器观察到的事件交给 attempt

按解析顺序调用 `try_admit`：

| 解析器观察到的内容 | 对应测量 |
| --- | --- |
| `null` | `JsonMeasurement::Null { depth }` |
| 布尔值 | `JsonMeasurement::Boolean { depth }` |
| 字符串 | `JsonMeasurement::String { depth, bytes }` |
| 数字 | `JsonMeasurement::Number { depth, bytes }` |
| 数组 | `JsonMeasurement::Array { depth, items }` |
| 对象 | `JsonMeasurement::Object { depth, entries }` |
| 对象键 | `JsonMeasurement::Key { bytes }` |

一次接纳会先检查数量转换和点上限，再检查累计节点数与 payload 容量。失败时，这次
接纳不会改变 attempt。流式解析器若想在进入下一个子节点前预检，可在 value transaction
上调用 `check_container_count`；该调用不会修改状态。

### 3. 只在完整 value 成功后提交

解析、规范化、校验和当前 value 边界内的其他工作全部成功后，再调用 `commit()`。如果
attempt 因 `?`、作用域结束或 panic unwind 被丢弃，只有暂存的 value 用量会回滚。

输入规则不同：decoder 一旦接受了原始或规范化字节，它们会立即入账。处理 stream 时，
通常为每个顶层 value 新建一个 attempt。这样后续 value 失败时，只会回滚自身的结构和
payload 用量，不会影响之前已提交的 value 或已接受的输入。

| 记账对象 | 何时正式入账 |
| --- | --- |
| 解码时接受的原始和规范化输入 | 接受时立即入账 |
| 增量 writer 已接受的输出 | 每个已接受前缀立即入账 |
| value 节点和 payload | 仅在 `commit()` 成功时入账 |

attempt 是记账边界，并不是通用的副作用回滚机制。它无法撤销已写出的内容、回调、
Hasher 更新或其他对象修改。

## 直接使用核心类型

对于一次独立测量，使用点上限：

```rust
use qubit_budget::ResourceLimit;

let depth = ResourceLimit::new("message-depth", 3_usize);
depth.check(3).expect("the inclusive maximum fits");

let error = depth.check(4).expect_err("depth four is rejected");
assert_eq!(error.exact_observed(), Some(4));
assert_eq!(error.maximum(), 3);
```

不能归还的容量使用预算；预算组的计费是全有或全无：

```rust
use qubit_budget::ResourceBudget;

let mut request = ResourceBudget::new("request-bytes", 5_u64);
let mut tenant = ResourceBudget::new("tenant-bytes", 2_u64);

let error = ResourceBudget::try_consume_group(&mut [&mut request, &mut tenant], 3)
    .expect_err("the tenant limit rejects the charge");

assert_eq!(error.index(), 1);
assert_eq!(request.remaining(), 5);
assert_eq!(tenant.remaining(), 2);
```

只有会显式归还的容量才使用资源池：

```rust
use qubit_budget::ResourcePool;

let mut files = ResourcePool::new("open-files", 2_u64);
files.try_acquire(2).expect("both slots are available");
files.release(1).expect("one slot is returned");

assert_eq!(files.available(), 1);
assert_eq!(files.in_use(), 1);
```

`consume_available` 是刻意提供的部分消费操作：它会消耗请求量和剩余额度中较小的值，
并返回实际消耗量。务必使用该返回值。

## 进阶选择

- 多个适配器要共享调用方拥有的预算时，使用 `JsonDecodeSession::borrowing_value`、
  `borrowing_input` 或 `borrowing_all`。
- 编码得到完整 `Vec<u8>` 时，先序列化成功再为输出计费；增量 writer 则为每段已接受
  的前缀立即计费。
- `ResourceBudget::try_write_string` 会先缓冲输出，渲染成功且 UTF-8 校验通过后才扣减
  字节预算。
- `StructureLimits`、`StructureBudget` 可限制非 JSON 的深度、节点、容器大小和键字节数；
  `StringLimits`、数值限制、`DurationBudget`、`TimeBudget` 覆盖相应场景。

## 错误与诊断

请使用类型化 accessor，不要解析错误文本：

| 错误 | 含义 | 拒绝后的状态 |
| --- | --- | --- |
| `LimitExceededError` | 单次测量超过包含式上限 | 不变 |
| `InsufficientBudgetError` | 预算计费或资源池获取量超出容量 | 不变 |
| `BudgetGroupError` | 预算组成员拒绝了计费 | 所有成员均不扣减 |
| `QuantityConversionError` | 原生测量值无法精确表示为 `Q` | 不变 |
| `MeasuredBudgetError` | 测量 API 返回的转换或记账错误 | 本次接纳不变 |
| `ResourceReleaseError` | 释放量超过 `in_use` | 不变 |

`Observation::Exact` 表示精确测量值。若集成层只能证明“已经超过上限”，
`Observation::AtLeast` 则提供一个安全下界。

## 排障与限制

| 现象 | 先检查什么 |
| --- | --- |
| 限制看起来没有生效 | 是否配置了该维度，适配层是否发出了对应测量？ |
| JSON value 用量一直为零 | 先检查 attempt，再在完整 value 成功后调用 `commit()`。 |
| 出错后输入或输出仍被计费 | 已接受的 I/O 本就不会回滚；只有暂存的 value 用量会回滚。 |
| 出现 `MeasuredBudgetError::Quantity` | 选择更宽的无符号 `Q`，不要先强制转换截断。 |
| 资源池释放失败 | 比较 `requested()` 与 `in_use()`，并为每次成功获取配对释放。 |

具体限额必须由应用层选择，crate 没有适用于所有业务的安全默认值。在不可信边界应
配置所有关键维度；仅限制原始输入无法限制规范化膨胀、嵌套、节点、payload 或输出。
跨线程共享可变预算时必须由外部同步。crate 的记账状态是固定大小，但不会限制解析器、
序列化器或应用自身产生的分配和副作用。

## 延伸阅读

- [中文 README](../README.zh_CN.md)
- [English user guide](user_guide.md)
- [API 文档](https://docs.rs/qubit-budget)
- [crates.io 上的 `qubit-json`](https://crates.io/crates/qubit-json)
