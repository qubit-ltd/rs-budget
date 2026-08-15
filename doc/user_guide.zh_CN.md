# qubit-budget 用户手册

[English user guide](user_guide.md) · [中文 README](../README.zh_CN.md) ·
[API 文档](https://docs.rs/qubit-budget)

本手册适用于 `qubit-budget` 0.4.x，要求 Rust 1.94 或更高版本。

## 手册目标与读者

当解析器、序列化器、转换器或 I/O 边界需要限制有限资源，并在拒绝操作时给出
准确原因，可以使用本 crate。本手册主要面向库作者和服务开发者，帮助他们建立
可复用的记账规则，同时避免把规则绑定到某一种解析器或异步运行时。

本 crate 只负责限制、可变记账和结构化错误。集成层仍需测量事件、执行 I/O、
选择具体限额，并决定发生错误后是重试、拒绝还是恢复。

## 概念模型

每项约束都由调用方定义的资源标识 `R` 和资源量 `Q` 组成：

- `R` 会保留在错误中，应用可以据此区分请求字节、JSON 节点、打开文件数等资源。
- `Q` 必须是精确的无符号整数，可选 `u8`、`u16`、`u32`、`u64`、`u128` 或
  `usize`。测量 API 会检查原生 `usize` 与 `u64` 的转换，绝不静默截断。
- 未配置的维度使用 `None` 表示。只要创建了 budget 或 pool，就意味着存在一项
  有限约束。

应根据资源的生命周期选择状态模型：

| 需求 | 类型 | 操作成功后 | 操作失败后 |
| --- | --- | --- | --- |
| 校验一个相互独立的值 | `ResourceLimit` | 不修改状态 | 返回测量值和上限 |
| 消耗不可返还的额度 | `ResourceBudget` | 扣减 `remaining` | 预算保持不变 |
| 借用并归还容量 | `ResourcePool` | 获取或释放会改变 `available` | 资源池保持不变 |

`ResourceBudget` 有意不实现 `Clone`，因为复制会把一份有限额度变成两份。
`ResourcePool` 不提供同步、等待、公平性或 RAII permit，也不会自动归还资源。

组合类型又定义了两层边界：

- `StructureLimits` 把深度、容器大小等点上限与累计节点预算组合起来。
- JSON session 会立即记录原始输入、规范化输入和 writer 已接受的输出；结构与
  payload 用量则先放在 `JsonValueTransaction` 中，调用 `commit` 后才生效。

## 贯穿场景：保护不可信 JSON 请求

假设网关每次接收一个 JSON 文档，验收标准如下：

- 原始输入与规范化输入各不超过 64 字节；
- 根节点计为第 1 层，最大深度为 3；
- 全文最多包含 8 个节点；
- 单个字符串最多 16 个 UTF-8 字节，累计 payload 最多 32 字节；
- 只有解析器完整处理文档后，value 用量才正式入账。

`qubit-budget` 不会解析这些字节。解析器或适配层需要在接受输入时计费，并在遍历
value 的过程中产生 `JsonMeasurement` 事件。

## 安装与 feature 选择

核心的 limit、budget、pool、结构、字符串和时长类型不需要启用 feature：

```toml
[dependencies]
qubit-budget = "0.4"
```

贯穿场景需要启用 JSON 记账：

```toml
[dependencies]
qubit-budget = { version = "0.4", features = ["json"] }
```

可选 feature 如下：

| Feature | 提供的公开能力 |
| --- | --- |
| `json` | JSON 资源、限制、value transaction 以及 decode/encode session |
| `big-integer` | 面向 `num_bigint::BigInt` 的 `BigIntegerLimits` |
| `big-decimal` | `BigDecimalLimits`，并同时启用 `big-integer` |
| `time` | 基于时钟的 `TimeBudget` 与 `TimeBudgetError` |

## 核心工作流

### 1. 配置自持有的 decode session

`JsonDecodeLimits::<JsonResource, usize>::new()` 创建的配置没有任何限制。只添加当前边界真正需要执行的
维度：

```rust
use qubit_budget::json::JsonDecodeLimits;
use qubit_budget::json::JsonDecodeSession;
use qubit_budget::json::JsonMeasurement;

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
assert_eq!(attempt.used_payload_bytes(), Some(5));
attempt.commit();

assert_eq!(session.input_budget().expect("configured input").used(), 7);
assert_eq!(session.value_budget().used_nodes(), Some(1));
# Ok::<(), qubit_budget::MeasuredBudgetError<qubit_budget::json::JsonResource, usize>>(())
```

深度包含根节点。`String::bytes` 和对象键长度使用 UTF-8 字节数；数字的字节数由
适配层按实际看到的数字表示计算。`PayloadBytes` 累计对象键、字符串和数字的字节数。

### 2. 按解析顺序接纳事件

每次遇到 JSON 事件都调用 `try_admit`：

| 解析事件 | 对应测量 |
| --- | --- |
| `null` | `JsonMeasurement::Null { depth }` |
| 布尔值 | `JsonMeasurement::Boolean { depth }` |
| 字符串 | `JsonMeasurement::String { depth, bytes }` |
| 数字 | `JsonMeasurement::Number { depth, bytes }` |
| 数组 | `JsonMeasurement::Array { depth, items }` |
| 对象 | `JsonMeasurement::Object { depth, entries }` |
| 对象键 | `JsonMeasurement::Key { bytes }` |

每次接纳会依次检查数量转换、点上限、累计节点数和累计 payload。只要其中一步失败，
本次事件就不会改变 transaction；如果业务流程能够妥善处理这次拒绝，仍可继续使用
同一个 transaction。

流式解析器如果要在进入下一个子节点前做预检，可调用
`check_container_count(JsonContainerKind::Sequence, prospective)`；对象使用 `Map`
变体。该检查不会修改状态。

### 3. 只提交完整 value

解析、规范化、校验以及当前 value 边界内的其他工作全部成功后，再调用 `commit`。
如果 attempt 被普通 drop、`?` 提前返回或 panic unwind 丢弃，暂存的 value 用量都会
回滚。

输入记账有意采用不同规则。attempt 一旦接受原始输入或规范化输入，即使之后丢弃
value transaction，这些计费也会保留，因为 decoder 已经执行了相应工作。

处理 JSON stream 时，通常为每个完整顶层 value 创建一个 attempt。这样后续 value
失败时只会回滚自身的 value 用量；之前提交的 value 和所有已接受输入仍保留在 session
中。

调用者通过 `begin_value()` 显式创建每个 attempt。

### 原子性矩阵

| 场景 | input | normalized input | value | output |
| --- | --- | --- | --- | --- |
| strict decode 成功 | 保留 | 不适用 | 提交 | 不适用 |
| strict decode 失败 | 保留 | 不适用 | 回滚 | 不适用 |
| lenient decode 失败 | 保留 | 保留 | 回滚 | 不适用 |
| 缓冲的 `Vec<u8>` output 失败 | 不适用 | 不适用 | 回滚 | 只在成功时计费；没有 `Vec` 就不计 output |
| buffered writer 部分失败 | 不适用 | 不适用 | 回滚 | 每个 accepted prefix 立即保留 |
| incremental writer 失败 | 不适用 | 不适用 | 回滚 | 每个 accepted prefix 立即保留 |
| stream 中单个 value 失败 | 跨 value 持续累计 | 跨 value 持续累计 | 仅当前 value 回滚 | 之前接受的 output 继续保留 |

raw input 和 normalized input 都会立即计费。丢弃 transaction 无法撤销 accepted
prefix、callback 副作用、`Hasher` 更新或对象 mutation。higher-level 操作可以有意让
同一个 transaction 覆盖更宽的业务边界，但暂存 value 状态仍然只能通过 `commit`
发布。

## 直接使用核心原语

### 点上限

单次消息深度等相互独立的测量适合使用 `ResourceLimit`：

```rust
use qubit_budget::ResourceLimit;

let depth = ResourceLimit::new("message-depth", 3_usize);
depth.check(3).expect("the inclusive maximum fits");

let error = depth.check(4).expect_err("depth four is rejected");
assert_eq!(error.resource(), &"message-depth");
assert_eq!(error.exact_observed(), Some(4));
assert_eq!(error.maximum(), 3);
```

### 累计预算与预算组

资源消耗后不能返还时，使用 `ResourceBudget`。预算组会先检查全部成员，确认都能
接受后才统一扣减：

```rust
use qubit_budget::ResourceBudget;

let mut request = ResourceBudget::new("request-bytes", 5_u64);
let mut tenant = ResourceBudget::new("tenant-bytes", 2_u64);

let error = ResourceBudget::try_consume_group(
    &mut [&mut request, &mut tenant],
    3,
)
.expect_err("the tenant budget rejects the charge");

assert_eq!(error.index(), 1);
assert_eq!(request.remaining(), 5);
assert_eq!(tenant.remaining(), 2);
```

`consume_available` 是有意设计的部分消费操作：它会扣除
`min(requested, remaining)`，并返回实际消费量。调用方必须使用返回值。

### 可释放容量

只有调用方会显式归还容量时，才使用 `ResourcePool`：

```rust
use qubit_budget::ResourcePool;

let mut files = ResourcePool::new("open-files", 2_u64);
files.try_acquire(2).expect("both slots are available");
files.release(1).expect("one acquired slot can be returned");

assert_eq!(files.available(), 1);
assert_eq!(files.in_use(), 1);
```

如果释放量大于 `in_use`，操作会返回 `ResourceReleaseError`，且资源池保持不变。

## 进阶用法

### 在多个集成之间共享预算

自持有 session 适合一次独立操作。如果多个适配器需要共同消耗调用方的预算，可使用
`borrowing_input`、`borrowing_all`、`borrowing_output` 或 `borrowing_value`。
此时真正的记账生命周期由调用方决定，例如让一次请求 stream 中的所有 value 共用
同一份预算。

### 为编码选择正确策略

`JsonEncodeSession` 把 output 记账和 value 记账分开：

- 如果序列化器返回完整 `Vec<u8>`，应先完成序列化，再调用
  `check_output_bytes`，并且只在序列化成功后对完整长度计费。没有返回 `Vec` 就表示
  没有被接受的输出。
- 如果 writer 逐步接受输出，每个已接受前缀都应立即调用
  `try_consume_output_bytes`。后续发生序列化、I/O、预算或 panic 错误时，这些计费不会
  撤销。
- 两种方式都通过 `try_admit` 记录 value 事件，并且只在完整 value 成功后调用
  `commit`。

attempt 是 value 记账边界，并非通用副作用回滚机制。丢弃 attempt 无法撤销 writer
输出、callback、`Hasher` 更新或对象 mutation。

### 事务式生成字符串

`ResourceBudget::try_write_string` 会先缓冲 UTF-8 输出，只有渲染成功且 UTF-8 校验
通过后才扣减预算：

```rust
use std::fmt::Write as _;

use qubit_budget::ResourceBudget;

let mut output = ResourceBudget::new("output-bytes", 8_u64);
let rendered = output
    .try_write_string(|writer| {
        write!(writer.as_fmt(), "id={}", 42)
    })
    .expect("five bytes fit");

assert_eq!(rendered, "id=42");
assert_eq!(output.used(), 5);
```

闭包也可以改用 `writer.as_io()`。无论是预算拒绝、renderer 错误、分配失败、非法
UTF-8，还是长度或数量转换失败，原预算都不会改变。

### 其他可复用辅助类型

- `StructureLimits` 与 `StructureBudget` 可在不依赖 JSON 的情况下限制深度、累计
  节点数、单个 sequence 的 item 数、单个 map 的 entry 数和键字节数。
- `StringLimits` 校验一个字符串的 UTF-8 字节长度。
- 启用对应 feature 后，`BigIntegerLimits` 与 `BigDecimalLimits` 可校验数字表示属性。
- `DurationBudget` 记录调用方提供的时长额度。
- 启用 `time` 后，`TimeBudget` 通过 `qubit-clock` 检查已用时间与截止时间。

## 错误与诊断

| 错误 | 含义 | 失败后的状态 |
| --- | --- | --- |
| `LimitExceededError` | 一次测量超过包含式上限 | 不修改状态 |
| `InsufficientBudgetError` | 累计计费或 pool 获取量超过余额 | 不修改状态 |
| `BudgetGroupError` | 预算组中有成员拒绝全有或全无计费 | 所有成员都不扣减，`index()` 指向首个拒绝者 |
| `QuantityConversionError` | 原生测量值无法由 `Q` 精确表示 | 不修改状态 |
| `MeasuredBudgetError` | 为原生测量统一封装转换错误或预算错误 | 被拒绝的接纳不修改状态 |
| `ResourceReleaseError` | pool 释放量超过当前 `in_use` | 不修改状态 |
| `BudgetedStringError` | 渲染、分配、UTF-8、长度、转换或预算失败 | 字符串预算不变 |

诊断时应使用类型化 accessor，不要解析 `Display` 文本。点上限错误和累计预算错误都
保留资源标识。`Observation::Exact` 表示精确测量；如果集成层只能确定测量已经越过
上限，则可用 `Observation::AtLeast` 表示安全下界。

## 排障

### 配置的限制没有生效

先确认该维度确实已配置。`empty()` 和 `unconfigured()` 创建的配置不会包含任何可选
限制。再检查集成层是否产生了对应测量；本 crate 无法自行观察 parser 或 writer 的
行为。

### JSON value 用量一直为零

`try_admit` 只修改 transaction 的 working state。完整 value 成功后必须对 attempt
调用 `commit`。提交前可检查 `attempt.used_nodes()`，提交后再检查
`session.value_budget().used_nodes()`。

### 出错后 input 或 output 用量没有回滚

这是已接受 I/O 的预期行为。decode attempt 会保留原始输入和规范化输入计费；
writer-oriented encode attempt 会保留 writer 已接受的输出前缀。drop 时只有暂存的
value 记账回滚。

### 返回 `MeasuredBudgetError::Quantity`

原生 `usize` 或 `u64` 测量无法由当前 `Q` 表示。应改用更宽的无符号数量类型，或者
降低输入测量值；不要先强制转换并截断，再交给预算检查。

### pool 释放失败

比较错误中的 `requested()` 与 `in_use()`。`ResourcePool` 不跟踪所有权，也不会自动
返还容量，因此集成层必须为成功 acquire 配对合法的显式 release。

## 限制与最佳实践

- 具体限额应由应用层决定；本 crate 不提供适用于所有场景的“安全默认值”。
- 面对不可信输入时，应配置所有真正重要的维度。仅限制原始输入，无法约束规范化
  膨胀、嵌套深度、节点数、payload 或输出。
- 资源标识应稳定且含义明确，日志和指标才能直接按资源聚合，而不必解析错误文本。
- `check_*` 用于预检，`try_*` 用于修改状态。如果其他代码可以同时改动同一记账
  对象，不能把预检结果当作预留额度。
- 如需跨线程共享可变预算，必须在外部同步。本 crate 定义记账语义，不提供并发协议。
- JSON transaction 的记账状态是固定大小，但它不会限制周边 parser、serializer 或
  应用执行的分配和副作用。
- 每个 attempt 应对应一个明确的回滚边界，并在集成文档中写清哪些外部效果会立即
  生效。

## 延伸阅读

- [中文 README](../README.zh_CN.md)
- [English user guide](user_guide.md)
- [API 文档](https://docs.rs/qubit-budget)
- [crates.io 上的 `qubit-json`](https://crates.io/crates/qubit-json)
- [源码仓库](https://github.com/qubit-ltd/rs-budget)
