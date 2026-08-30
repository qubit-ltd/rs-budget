# qubit-budget 用户手册

[English user guide](user_guide.md) · [中文 README](../README.zh_CN.md) ·
[API 文档](https://docs.rs/qubit-budget)

本手册适用于 `qubit-budget` 0.4.x 和 Rust 1.94 或更高版本。文中的例子提炼自
Qubit crate 的真实用法，只省略了与计费无关的业务代码。

## 手册目标与读者

在一个边界上，“太大”可能有完全不同的含义：HTTP 响应的字节数过多，配置树层级过深，
目录遍历同时打开的句柄过多，或者重试次数和耗时过长。只有一个计数器并不足够：一次
失败是否改变了状态？究竟是哪一项限制拒绝了操作？

`qubit-budget` 面向库和服务的开发者，提供小而独立的记账原语。调用方仍需决定测量
什么、边界在哪里，以及如何把类型化错误转换为应用错误。

## 概念模型

### Qubit 中的典型集成方式

| crate | 受保护的工作 | 使用的原语 | 为什么适合 |
| --- | --- | --- | --- |
| `qubit-http` | 从流式 HTTP 响应收集的字节 | `ResourceBudget` | 每个已接受的分块都会永久消耗响应体额度。 |
| `qubit-config` | 配置源的字节、属性、节点和包含的子源 | `ResourceBudget`、`ResourceLimit`、预算组 | 子源必须同时满足自身和所有父源的限制。 |
| `qubit-local-files` | 复制或遍历时打开的目录读取器 | `ManagedResourcePool` | 读取器打开期间持有 permit，Drop 自动归还容量。 |
| `qubit-retry` | 重试次数、累计操作耗时和总耗时 | `ResourceBudget`、`DurationBudget`、`TimeBudget` | 三者的生命周期不同，必须分别记账。 |
| `qubit-json` | 原始输入、规范化输入和一个 JSON value | JSON session 与 transaction | 已接受的 I/O 必须保留计费；失败的 value 不能占用结构容量。 |

### 记账模式

| 需求 | 类型 | 成功后 | 失败后 |
| --- | --- | --- | --- |
| 校验一个事实，不消耗容量 | `ResourceLimit` | 不修改状态 | 不修改状态；错误包含测量值和上限。 |
| 消耗不会归还的容量 | `ResourceBudget` | `used` 增加，`remaining` 减少 | 预算不变。 |
| 记录通用结构 | `StructureBudget` | 每个已接受的测量立即扣减 | 只有被拒绝的测量不改变状态。 |
| 多项限制必须作为一次决策 | `ResourceBudget::try_consume_group` | 所有预算一起扣减 | 所有预算都不扣减。 |
| 通过显式 release 复用容量 | `ResourcePool` | 获取或释放会改变占用量 | 资源池不变。 |
| 通过所有权生命周期复用容量 | `ManagedResourcePool` | 返回基于 Drop 的 permit | 资源池不变。 |
| 限制时间 | `DurationBudget` 或 `TimeBudget` | 记录已消耗时长或检查时钟截止时间 | 被拒绝的检查不改变状态。 |

资源标识 `R` 应当能直接用于错误和指标，例如 `"response body"` 或业务枚举。
数量 `Q` 是精确的无符号整数（`u8` 到 `u128`，或 `usize`）。可选限制为 `None`
时，表示该维度没有配置，而不是存在一份隐藏的无限预算。

## 安装与最小配置

核心的 limit、budget 和 pool 不需要开启 feature，添加依赖即可：

```toml
[dependencies]
qubit-budget = "0.4"
```

只为实际使用的集成能力启用 feature。例如，JSON value 会话需要 `json` feature：

```toml
[dependencies]
qubit-budget = { version = "0.4", features = ["json"] }
```

本 crate 支持 Rust 1.94 及更高版本。可选 feature 为 `json`、`big-integer`、
`big-decimal`（会同时启用 `big-integer`）和 `time`。

## 贯穿场景与核心工作流：读取有上限的响应体

`qubit-http` 若发现 `Content-Length` 明显超限，会先拒绝该响应；但服务器可能没有这个
头，也可能谎报，所以读取分块时仍需要一份预算。下面是
`HttpResponse::read_body` 的核心逻辑，省略了网络和错误映射：

其中 `response_chunks` 表示应用 HTTP 客户端提供的分块。全部分块被接受即表示成功，
此时 `body` 包含它们按顺序拼接的内容；一旦拒绝，当前分块既不会写入 `body`，也不会
改变预算。

```rust
use qubit_budget::ResourceBudget;

let body_limit = 1_048_576_usize;
let mut body = Vec::new();
let mut body_budget = ResourceBudget::new("response body", body_limit);

for chunk in response_chunks {
    body_budget.try_consume(chunk.len())?;
    body.extend_from_slice(&chunk);
}
# Ok::<(), qubit_budget::InsufficientBudgetError<&str, usize>>(())
```

顺序非常重要：先执行 `try_consume`，再把分块追加到响应体。如果一个分块会越过上限，
这一分块既不会改变预算，也不会进入已收集的响应体。`qubit-http` 会使用
`used() + requested()` 报告触发拒绝时观察到的总大小。

这是**累计预算**。不能改用 `ResourceLimit`：每个分块都可能很小，但它们的总和仍可能
过大。

## 进阶用法

### 场景二：同时执行本地和父级限制

配置源可以包含其他配置源。`qubit-config` 为每个源维护字节数、属性数、节点数和子源数
预算，同时借用全部祖先的对应预算。子源新增一个属性时，所有处于生效范围内的预算都
必须接受，否则一个都不能扣减：

```rust
use qubit_budget::ResourceBudget;

let mut root_properties = ResourceBudget::new("root properties", 100_usize);
let mut child_properties = ResourceBudget::new("child properties", 10_usize);

ResourceBudget::try_consume_group(
    &mut [&mut root_properties, &mut child_properties],
    1,
)?;
# Ok::<(), qubit_budget::BudgetGroupError<&str, usize>>(())
```

如果子源没有余额，根源预算不会被误扣。错误可通过 `index()` 找到第一个拒绝者，再通过
`source_error()` 获取原始的 `InsufficientBudgetError`。

同一会话还用 `ResourceLimit` 检查嵌套深度。深度是当前节点的属性，不是每处理一个
后续节点都要消耗的资源：

```rust
use qubit_budget::ResourceLimit;

let depth = ResourceLimit::new("nesting depth", 16_usize);
depth.check(current_depth)?;
# Ok::<(), qubit_budget::LimitExceededError<&str, usize>>(())
```

### 场景三：复用目录句柄容量

复制目录树时可能同时打开多个目录读取器。`qubit-local-files` 用 `ManagedResourcePool`
管理它们：每个读取器持有一个 permit，因此正常结束、错误返回和 panic unwinding 都会归还容量：

```rust
use qubit_budget::ManagedResourcePool;

let directories = ManagedResourcePool::new("open directory", 32_usize);
let permit = directories.try_acquire(1)?;

// Read this directory and possibly descend into it.

drop(permit);
# Ok::<(), Box<dyn std::error::Error>>(())
```

与 `ResourceBudget` 不同，permit 被 Drop 后容量会回来。目录遍历器还可以选择 `Reopen`
策略：资源池满时先关闭保留的读取器、Drop 对应 permit，再重试获取。

需要显式、受检查的 `release` 调用时使用 `ResourcePool`；需要让所有权自动归还容量时
使用 `ManagedResourcePool`。两者都表示有限容量，均不会等待容量或保证公平性。

### 场景四：同时维护三种不同的限制

`qubit-retry` 的 `RetryBudget` 组合了三种原语：

1. `ResourceBudget<RetryResource, u32>` 用于接纳有限次数的尝试。
2. `DurationBudget` 记录全部已完成操作累计消耗的时间。
3. `TimeBudget` 通过 `qubit-clock` 执行一个端到端的单调时钟截止时间，其中包括退避和
   观察器工作。

`qubit-retry` 的公开 API 通过一个 `RetryBudget` 暴露这三项限制：

```rust
use std::time::Duration;

use qubit_clock::StdMonotonicClock;
use qubit_retry::RetryBudget;
use qubit_retry::RetryPolicy;

let clock = StdMonotonicClock::new();
let policy = RetryPolicy::builder()
    .max_attempts(3)
    .max_operation_elapsed(Duration::from_secs(10))
    .max_total_elapsed(Duration::from_secs(30))
    .build()?;
let mut budget = RetryBudget::new(&clock, *policy.limits())?;

let attempt = budget.begin_attempt()?;
// Run one request attempt here.
let snapshot = budget.finish_attempt(attempt);

budget.check_retry_after(Duration::from_millis(500))?;
assert_eq!(snapshot.attempts(), 1);
# Ok::<(), Box<dyn std::error::Error>>(())
```

`begin_attempt()` 会先检查所有后续限制，再扣减一次尝试额度。`finish_attempt()` 会记录
实际操作耗时，即使这次操作已经超过时长额度也一样。`check_retry_after()` 会在休眠前
用总截止时间检查计划中的退避时长。

这种拆分是有意的。一项操作即使耗时过长，也可能已经完成；该事实需要被记录下来，
超出的时间会耗尽后续操作的时长额度，因此之后的 `begin_attempt()` 会拒绝重试。另一边，
`check_retry_after(delay)` 会拒绝任何会达到总截止时间的延迟。由此可见，时长额度与
时钟截止时间不能互相替代。

使用 `TimeBudget` 需开启 `time` feature：

```toml
[dependencies]
qubit-budget = { version = "0.4", features = ["time"] }
```

### 场景五：让一个已解码 JSON value 具备事务性

`qubit-json` 通过 `NormalizingJsonDecoder::decode_with_session` 规范化并解码不可信 JSON。
已执行的 I/O 工作必须保留计费，但如果解析或反序列化随后失败，该 value 的节点数和
payload 用量不能被消耗。

decoder 会创建 attempt，在规范化前计原始输入，在规范化期间计规范化输入，扫描时接纳
JSON 测量，只有反序列化成功后才调用 `commit()`。简化后的 session 配置如下：

```rust
use qubit_budget::json::JsonDecodeLimits;
use qubit_budget::json::JsonDecodeSession;
use qubit_budget::json::JsonResource;

let mut session = JsonDecodeSession::from_limits(
    JsonDecodeLimits::<JsonResource, usize>::builder()
        .max_input_bytes(64 * 1024)
        .max_normalized_input_bytes(64 * 1024)
        .max_depth(32)
        .max_nodes(10_000)
        .build(),
);
```

对于自行上报测量的适配层，value 部分的核心是：

```rust
use qubit_budget::json::JsonMeasurement;

let mut attempt = session.begin_value();
attempt.try_consume_input_bytes(raw.len())?;
attempt.try_consume_normalized_input_bytes(normalized.len())?;
attempt.try_admit(JsonMeasurement::String {
    depth: 1,
    bytes: 5,
})?;

// Parse, normalize, and deserialize the complete value here.
attempt.commit()?;
# Ok::<(), qubit_budget::MeasuredBudgetError<qubit_budget::json::JsonResource, usize>>(())
```

### 完整原子性矩阵

| 场景 | input | normalized input | value | output |
| --- | --- | --- | --- | --- |
| strict decode 成功 | 保留 | 不适用 | 提交 | 不适用 |
| strict decode 失败 | 保留 | 不适用 | 回滚 | 不适用 |
| lenient decode 失败 | 保留 | 保留 | 回滚 | 不适用 |
| 缓冲的 `Vec<u8>` output 失败 | 不适用 | 不适用 | 回滚 | 只在成功时计费；没有 `Vec` 就不计 output |
| buffered writer 部分失败 | 不适用 | 不适用 | 回滚 | 每个 accepted prefix 立即保留 |
| incremental writer 失败 | 不适用 | 不适用 | 回滚 | 每个 accepted prefix 立即保留 |
| stream 中单个 value 失败 | 跨 value 累计 | 跨 value 累计 | 仅当前 value 回滚 | 之前接受的 output 继续保留 |

raw input 和 normalized input 会立即入账。丢弃 transaction 无法撤销 accepted prefix、
callback 副作用、`Hasher` 更新或对象 mutation。higher-level 操作可以有意让一个
transaction 覆盖更宽的业务边界，但暂存 value 用量只有 `commit` 才会发布。
调用者通过 `begin_value()` 显式创建每个 attempt。

第一次 value admission 失败会使 transaction 进入 poisoned 状态。之后的每次
admission 和 `commit` 都返回首次错误，poisoned transaction 的 `commit` 不会发布
暂存的 value 状态；丢弃它会回滚全部暂存用量。raw input、normalized input 或 writer
I/O 失败本身不会毒化 value transaction；已经接受的 I/O 计费和 output prefix 仍然
立即生效。

## 专用辅助类型

- `StructureLimits` 和 `StructureBudget` 将深度、序列元素数、映射条目数、键字节数等
  点上限与累计节点预算组合起来。
- `StringLimits` 检查一个字符串的 UTF-8 字节长度。
- `BigIntegerLimits` 与 `BigDecimalLimits` 是受 feature 控制的数值表示限制。
- `ResourceBudget::try_write_string` 会先缓冲生成的文本，只有渲染成功且 UTF-8 校验通过
  后才扣减字节预算。

## 错误与诊断

请使用类型化 accessor，不要解析 `Display` 文本。`LimitExceededError` 提供测量值和
包含式上限；`InsufficientBudgetError` 提供 `limit`、`remaining`、`requested`；
`ResourceReleaseError` 表示不合法的资源池归还。原生 `usize` 和 `u64` 测量 API 在
无法精确转换到 `Q` 时返回 `MeasuredBudgetError`，不要先强制转换并截断。

## 排障

- 如果被拒绝的操作仍改变了状态，先确认它是不是立即入账的工作（例如已接受的 input
  或 writer output），而不是暂存的 JSON value admission。
- 预算组扣减失败时，使用 `BudgetGroupError::index()` 和 `source_error()` 查看第一个
  拒绝请求的预算；不要改为逐个预算重试扣减。
- 如果原生 `usize` 或 `u64` 测量无法转换，保留原始数值用于诊断，并选择能精确表示
  应用取值范围的数量类型 `Q`。

## 限制与最佳实践

具体限额必须在应用层决定，crate 不提供通用的安全默认值。一个简单的判断法是：校验
事实用 limit；不可返还的工作用 budget；多个范围必须一起接受时用预算组；容量确实会
归还时用 pool；只有完整工作单元成功才能占用 value 容量时用 transaction。

本 crate 不解析数据、不执行 I/O、不等待资源池容量、不为 `ResourcePool` 提供同步、
不替应用选择限额，也不定义错误后的恢复策略。`ManagedResourcePool` 会同步自身的
记账并在 Drop 时归还 permit，但同样不会等待容量，也不保证公平性。

## 延伸阅读

- [中文 README](../README.zh_CN.md)
- [English user guide](user_guide.md)
- [API 文档](https://docs.rs/qubit-budget)
