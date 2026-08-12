# rs-budget

[![Rust CI](https://github.com/qubit-ltd/rs-budget/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-budget/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-budget/coverage-badge.json)](https://qubit-ltd.github.io/rs-budget/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-budget.svg?color=blue)](https://crates.io/crates/qubit-budget)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

`qubit-budget` 为需要处理不可信或潜在无界工作量的 Rust 库提供可组合的资源
约束。它把记账机制与 parser、wire protocol、transport、文件系统、重试执行器或
诊断渲染器拥有的策略分开：使用方决定资源标识、默认值和公开错误，`qubit-budget`
负责让一次操作中的限制和状态转换保持一致。

## 为什么资源限制必须可组合

对于结构化输入，只限制输入总字节数并不能解决全部问题。一个很小的文档仍可能
具有很深的递归、包含大量微小节点、通过超大对象或数组产生局部扇出、携带超长 key，
或者膨胀成更大的输出。这些观测值含义不同，不能悄悄共用一个计数器。

因此，一个可靠的边界同时需要点限制和累计预算：

- 点检查把当前深度、容器大小、key 长度、字符串长度或数字表示与包含边界在内的
  最大值比较。
- 累计预算覆盖整个操作；递归子节点或嵌套 adapter 不能重新获得父级的完整额度。
- 组合操作先完成所有点检查，再消耗节点额度；累计记账失败时余额保持不变。
- 记账层报告精确的资源事实；使用方决定何时观察数据，并在自己的边界转换错误。

## 安装

```toml
[dependencies]
qubit-budget = "0.4"
```

默认 feature 集为空，按需启用扩展：

| Feature | 提供内容 |
| --- | --- |
| `json` | 提供方向明确的 `JsonDecodeLimits`/`JsonEncodeLimits`、共享 `JsonValueLimits` 和操作会话 |
| `serde-json` | 提供带预算检查的 Serde JSON 序列化/反序列化 adapter，同时启用 `json` |
| `big-integer` | 提供 `BigIntegerLimits`，限制幅值 bit 数和有效十进制位数 |
| `big-decimal` | 提供 `BigDecimalLimits`，限制系数和 scale 幅值 |
| `time` | 单调时钟预算 `TimeBudget` 与 `TimeBudgetError`（`DurationBudget` 始终可用） |

最低支持的 Rust 版本为 1.94。

下面的 Serde JSON 端到端示例需要启用 `serde-json`，并由应用直接声明自己使用的
类型依赖：

```toml
[dependencies]
qubit-budget = { version = "0.4", features = ["serde-json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## 快速开始

假设一个 wire 边界既要接纳不可信 JSON 文档，又要输出大小受限的响应。解码和编码
使用不同方向的会话：只有解码会消耗输入字节，只有编码会消耗输出字节，两者可以
复用同一套 JSON value 限制。

```rust
use qubit_budget::decode_slice;
use qubit_budget::encode_to_vec;
use qubit_budget::JsonDecodeLimits;
use qubit_budget::JsonDecodeSession;
use qubit_budget::JsonEncodeLimits;
use qubit_budget::JsonEncodeSession;
use qubit_budget::JsonResource;
use qubit_budget::JsonValueLimits;
use qubit_budget::ResourceLimit;
use qubit_budget::StructureLimits;
use serde::Deserialize;
use serde::Serialize;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
#[derive(Debug, Deserialize, Serialize)]
struct Document {
    items: Vec<String>,
}

let input = br#"{"items":["alpha"]}"#;

let structure = StructureLimits::empty()
    .with_depth_limit(ResourceLimit::new(JsonResource::Depth, 64))
    .with_nodes_limit(ResourceLimit::new(JsonResource::Nodes, 100_000))
    .with_sequence_items_limit(ResourceLimit::new(
        JsonResource::SequenceItems,
        10_000,
    ))
    .with_map_entries_limit(ResourceLimit::new(
        JsonResource::MapEntries,
        10_000,
    ))
    .with_key_bytes_limit(ResourceLimit::new(JsonResource::KeyBytes, 256));
let value_limits = JsonValueLimits::empty()
    .with_structure_limits(structure)
    .with_string_bytes_limit(ResourceLimit::new(
        JsonResource::StringBytes,
        256 * 1024,
    ))
    .with_number_bytes_limit(ResourceLimit::new(JsonResource::NumberBytes, 4_096))
    .with_payload_bytes_limit(ResourceLimit::new(
        JsonResource::PayloadBytes,
        512 * 1024,
    ));

let decode_limits = JsonDecodeLimits::empty()
    .with_input_bytes_limit(ResourceLimit::new(
        JsonResource::InputBytes,
        1_048_576,
    ))
    .with_value_limits(value_limits);
let mut decode_session = JsonDecodeSession::owned(decode_limits);
let document: Document = decode_slice(input, &mut decode_session)?;

let encode_limits = JsonEncodeLimits::empty()
    .with_output_bytes_limit(ResourceLimit::new(
        JsonResource::OutputBytes,
        1_048_576,
    ))
    .with_value_limits(value_limits);
let mut encode_session = JsonEncodeSession::owned(encode_limits);
let output = encode_to_vec(&document, &mut encode_session)?;
assert_eq!(output, input);
# Ok(())
# }
```

`decode_slice` 先从调用方持有的 `JsonDecodeSession` 消耗完整输入长度，再执行词法
准入和类型化 Serde 解码。`encode_to_vec` 通过 `JsonEncodeSession` 在线累计结构与
输出字节；`encode_to_writer` 会先缓冲已通过检查的完整文档，再触碰外部 writer。
每个独立边界应创建新会话；重复使用同一会话则会有意跨调用累计输入/输出、节点和
payload 消耗。

当更大的操作已经持有方向性预算时，可使用这些输出和 value budget 构造
`JsonEncodeSession::borrowing`。编码器会原地记账，避免重复预遍历或脱离调用方的预算副本。

同样的状态规则也让失败处理保持明确。例如，节点上限为 1 时，第一次记账成功，
第二次记账失败，并且已接受的节点数不会被改变：

```rust
use qubit_budget::JsonResource;
use qubit_budget::JsonValueBudget;
use qubit_budget::JsonValueLimits;
use qubit_budget::ResourceLimit;
use qubit_budget::StructureLimits;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let structure = StructureLimits::empty()
    .with_nodes_limit(ResourceLimit::new(JsonResource::Nodes, 1));
let limits = JsonValueLimits::empty().with_structure_limits(structure);
let mut budget = JsonValueBudget::new(limits);
budget.charge_node()?;
assert!(budget.charge_node().is_err());
# Ok(())
# }
```

decoder 可以匹配 `BudgetError`，再转换成自身已有的领域错误；`rs-value` 就是把
JSON 资源事实转换为 `ValueWireDecodeError`。

## 从配置到会话

`StructureLimits<R, Q>` 是可复用的结构部分，包含深度、累计节点、序列条目、映射
条目和结构化 key 字节限制。`JsonValueLimits<R, Q>` 再增加单字符串、单数字和累计
payload 限制；`JsonDecodeLimits` 只增加输入字节，`JsonEncodeLimits` 只增加输出
字节。对应 session 持有一次操作的可变状态；同一份不可变 limits 可以构造任意多个
全新会话。

未配置的维度用 `Option::None` 表示，而不是创建一个需要在每次调用中驱动的“无限”
预算对象。点检查或累计消费失败不会回滚本次操作此前已经接受的消耗；但被拒绝的
检查或消费请求本身是原子的，不会改变对应维度的剩余额度。

## 下游如何使用

| 下游 crate | 实际用法 | 体现的价值 |
| --- | --- | --- |
| `rs-value` | 分别使用 `JsonDecodeLimits` 和 `JsonEncodeLimits`，让一个方向会话贯穿一次 wire 操作，再把 `BudgetError` 映射为自身 wire 错误。 | 读取与写出策略不会意外消耗错误方向的字节额度。 |
| `rs-redact` | 在嵌套诊断渲染器之间共享一次操作的输入、输出和掩码预算。 | 子组件不能静默重置外层操作的额度。 |
| `rs-config` | 在配置边界应用 JSON wire、source session、composite 聚合和插值预算。 | source 局部与聚合 charge 原子提交，普通 Config 反序列化也会限制已解码值。 |
| `rs-datatype` | 与类型化 value 共享借用的 JSON value 预算，并限制字符串/数字。 | 嵌套类型不能重置外层操作的 value 额度。 |
| `rs-http`、`rs-fs`、`rs-local-files` | 数据到达时为 response body、stream 和文件读取记账，包括长度未知的输入。 | 不同分块方式和 transport 错误不会破坏累计上限。 |
| `rs-metadata` | 使用方向明确的 session 限制 metadata JSON wire 操作。 | metadata 边界与应用 payload 具有相同的接纳和输出保证。 |
| `rs-retry` | 组合尝试次数 `ResourceBudget`、显式时长 `DurationBudget` 和连续 elapsed deadline。 | 同一个领域策略可以组合不同资源语义。 |

## 核心能力

| 需求 | 公开 API |
| --- | --- |
| 单次、包含边界的点检查 | `ResourceLimit<R, Q>` |
| 不可归还的累计消耗 | `ResourceBudget<R, Q>` |
| 多个累计预算的原子消耗 | `ResourceBudget::try_consume_group`、`BudgetGroupError<R, Q>` |
| 可获取和释放的容量 | `ResourcePool<R, Q>` |
| 点限制/累计预算失败事实 | `BudgetError<R, Q>`：`LimitExceeded`、`Insufficient` |
| 非法 pool 释放事实 | `ResourceReleaseError<R, Q>` |
| 通用嵌套数据限制 | `StructureLimits<R, Q>`、`StructureBudget<R, Q>` |
| JSON value 遍历限制（`json`） | `JsonValueLimits<R, Q>`、`JsonValueBudget<R, Q>` |
| 定向 JSON 操作（`json`） | `JsonDecodeLimits`/`JsonDecodeSession`、`JsonEncodeLimits`/`JsonEncodeSession` |
| Serde 已解码值的增量准入（`serde-json`） | `BudgetedJsonValueSeed<'_, R, Q>` |
| 显式时长限制 | `DurationBudget<R>` |
| 基于单调时钟的时间限制（`time`） | `TimeBudget<R, C>`、`TimeBudgetError` |

数量使用精确的无符号类型。通用资源预算保持泛型；结构化默认结构限制以及 JSON
value、Serde JSON 辅助 API 默认使用 `usize`，与 Rust 集合和字节长度一致。下游若需要
固定宽度语义，应显式指定 `Q`（例如 `JsonValueLimits<MyResource, u64>`）。
JSON 解码会话可以使用 `owned(...)`，也可以用 `borrowing(...)` 借用调用方持有的输入
和 value 预算。

`BudgetError::LimitExceeded` 在测量值明确时报告 `Observation::Exact`，只能得到安全
下界时报告 `Observation::AtLeast`；下游不能把 `AtLeast` 当成精确值。

非 JSON 文本也可以直接使用相同的稳定数量语义：

```rust
let limits = StringLimits::empty().with_utf8_bytes_limit(
    ResourceLimit::new(MyResource::TextBytes, 1024_u64),
);
limits.check(input)?;
```

`serde-json` adapter 对 `serde_json` 任意精度数字和 raw value 所使用的私有 serializer
形状保留了一个小型兼容层。它保持非递归，并在类型化解码前完成词法准入。仓库中的
`fuzz/` target 会把解码接纳结果与 `serde_json` 做差分比较，并检查 session 记账不变量；
请在 nightly toolchain 上使用 `cargo fuzz` 运行。

## 边界

- 核心记账 API 不解析 JSON、不执行 I/O、不分配输出，也不决定使用方何时检查观测值。
  可选的 `serde-json` feature 提供带预算检查的 Serde JSON adapter，但仍不替使用方
  选择限制或领域错误策略。
- 它不选择默认上限、重试策略、脱敏策略、调度方式或应用专属错误类型。
- `BudgetError` 描述的是机制事实，不是所有应用都必须暴露的统一领域错误。使用方
  应在自己的公开边界根据资源和值的变体完成转换。
- 释放量超过 `ResourcePool` 当前占用量不属于预算耗尽；`ResourcePool::release`
  返回独立的 `ResourceReleaseError`。
- `ResourcePool` 是有限且不提供同步的容量对象，不包含等待、公平性、permit、取消
  或并发访问机制。
- `DurationBudget` 只计算调用方显式提交的时长。`TimeBudget` 观察注入的单调时钟，
  因而 operation、等待、排队和 backoff 都会消耗同一个 deadline。

## 延伸阅读

- [英文用户指南](doc/user_guide.md)
- [中文用户指南](doc/user_guide.zh_CN.md)
- [API 文档](https://docs.rs/qubit-budget)
- [English README](README.md)

## 测试

```bash
# 使用默认 feature 集运行测试
cargo test

# 使用项目声明的全部 feature 运行测试
cargo test --all-features

# 运行项目 CI 检查
./ci-check.sh

# 检查代码覆盖率
./coverage.sh
```

## 许可证

Copyright (c) 2025 - 2026. Haixing Hu. All rights reserved.

本项目基于 Apache License 2.0 授权。完整许可证文本请参阅
[LICENSE](LICENSE)。

## 贡献

欢迎贡献。请遵循 Rust API 指南，及时更新公共 API 文档与测试，并在提交
Pull Request 前运行 `./align-ci.sh` 格式化代码，运行 `./ci-check.sh` 对齐 CI 要求。

## 作者

**Haixing Hu** - *Qubit Co. Ltd.*

仓库地址：[https://github.com/qubit-ltd/rs-budget](https://github.com/qubit-ltd/rs-budget)
