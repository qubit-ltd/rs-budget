# `qubit-budget` 用户指南

[English guide](user_guide.md) | [README](../README.zh_CN.md) | [API 文档](https://docs.rs/qubit-budget)

本手册适用于 `qubit-budget` 0.4，面向需要为 parser、decoder、encoder 或遍历过程
设置明确资源边界，同时保留自身资源名称和公开错误模型的库作者。

## 概念模型

`ResourceLimit<R, Q>` 表示包含边界的单次观测上限；`ResourceBudget<R, Q>` 记录
不可归还的累计消耗；`ResourcePool<R, Q>` 表示可复用容量。点限制失败返回
`BudgetError::LimitExceeded`，累计请求超过余额返回 `BudgetError::Insufficient`。
释放量超过 pool 当前占用量时返回独立的 `ResourceReleaseError`，而不是
`BudgetError`。

JSON 记账分为三层：

- `JsonValueLimits` 与 `JsonValueBudget` 负责与方向无关的结构、节点、key、字符串、
  数字和累计 payload。
- `JsonDecodeLimits` 与 `JsonDecodeSession` 只增加累计输入字节。
- `JsonEncodeLimits` 与 `JsonEncodeSession` 只增加累计输出字节。

Limits 是不可变配置，session 是一次操作的可变状态；未配置的维度使用
`Option::None` 表示。

默认的 `StructureLimits` 与 `StructureBudget` 使用 `usize`，与 Rust 集合长度和数量
一致。JSON value、字符串和大数辅助 API 使用 `u64`，以便跨平台稳定表达 wire 测量值。

## 贯穿场景：接纳请求并限制响应

假设一个 endpoint 接收小型 JSON 请求并返回紧凑 JSON。成功标准是：在类型化解码
前完成输入准入，并保证完整响应不超过输出策略。

## 安装与最小配置

```toml
[dependencies]
qubit-budget = { version = "0.4", features = ["serde-json"] }
serde = { version = "1.0", features = ["derive"] }
```

先构造一份共享 value 策略，再把它分别放入解码和编码限制：

```rust
use qubit_budget::JsonDecodeLimits;
use qubit_budget::JsonEncodeLimits;
use qubit_budget::JsonResource;
use qubit_budget::JsonValueLimits;
use qubit_budget::ResourceLimit;
use qubit_budget::StructureLimits;

let structure = StructureLimits::empty()
    .with_depth_limit(ResourceLimit::new(JsonResource::Depth, 8))
    .with_nodes_limit(ResourceLimit::new(JsonResource::Nodes, 128));
let values = JsonValueLimits::empty()
    .with_structure_limits(structure)
    .with_payload_bytes_limit(ResourceLimit::new(
        JsonResource::PayloadBytes,
        4096,
    ));
let decode_limits = JsonDecodeLimits::empty()
    .with_input_bytes_limit(ResourceLimit::new(JsonResource::InputBytes, 4096))
    .with_value_limits(values);
let encode_limits = JsonEncodeLimits::empty()
    .with_output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, 4096))
    .with_value_limits(values);
```

## 核心工作流

调用方持有 session，并把它传给 Serde adapter。这就是外部准入边界：
`decode_slice` 会在词法检查和类型化反序列化之前消耗完整输入字节；
`encode_to_vec` 在生成紧凑输出时检查 value 并累计字节。

```rust
use qubit_budget::decode_slice;
use qubit_budget::encode_to_vec;
use qubit_budget::JsonDecodeLimits;
use qubit_budget::JsonDecodeSession;
use qubit_budget::JsonEncodeLimits;
use qubit_budget::JsonEncodeSession;
use qubit_budget::JsonResource;
use qubit_budget::ResourceLimit;
use serde::Deserialize;
use serde::Serialize;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
#[derive(Debug, Deserialize, Serialize)]
struct Request {
    name: String,
}

let input = br#"{"name":"Ada"}"#;
let decode_limits = JsonDecodeLimits::empty().with_input_bytes_limit(
    ResourceLimit::new(JsonResource::InputBytes, 64),
);
let mut decode_session = JsonDecodeSession::owned(decode_limits);
let request: Request = decode_slice(input, &mut decode_session)?;

let encode_limits = JsonEncodeLimits::empty().with_output_bytes_limit(
    ResourceLimit::new(JsonResource::OutputBytes, 64),
);
let mut encode_session = JsonEncodeSession::owned(encode_limits);
let output = encode_to_vec(&request, &mut encode_session)?;
assert_eq!(output, input);
# Ok(())
# }
```

目标实现 `Write` 时可使用 `encode_to_writer`。预算或 Serde 失败发生时，adapter
尚未触碰目标 writer；最后一次 `write_all` 如果发生 I/O 失败，仍可能留下部分输出，
因为 `Write` 没有回滚操作。

## 进阶用法

只启用 `json` feature 时，可以由其他 parser 驱动 `JsonValueBudget` 和定向 session。
非 JSON 嵌套数据使用 `StructureLimits`；单维资源使用 `ResourceLimit`、
`ResourceBudget` 或 `ResourcePool`。`DurationBudget` 始终可用；`time` feature 另外
提供单调时钟 `TimeBudget` 及其错误类型。

## 错误与诊断

本 crate 报告记账事实，不规定应用级错误策略；请在领域边界转换 `BudgetError` 或
`ResourceReleaseError`。被拒绝的点检查或累计请求不会改变对应维度，但本次操作
此前已经接受的消耗不会回滚。因此，即使后续词法检查或类型化解码失败，
`decode_slice` 已消费的输入字节仍会保留。

## 排障

- 第二份文档意外失败时，先检查是否复用了同一个 session；独立操作应新建 session。
- 深度检查通过但节点耗尽时，注意深度是点限制，而节点按会话累计。
- 外部 writer 在失败后留下前缀时，应检查 I/O 错误；预算和 Serde 失败发生在最终
  写出之前。

## 限制与最佳实践

本 crate 不决定默认字节、节点、深度、重试或脱敏策略，也不定义应用的公开错误类型。
请在拥有策略的边界配置 limits。不可变 limits 可以复用；独立操作应创建新 session；
只有确实需要跨调用累计时才复用 session。

`serde-json` adapter 对 `serde_json` 任意精度数字和 raw value 所使用的私有 serializer
形状保留了一个小型兼容层，并在类型化解码前执行非递归词法预检。仓库提供了用于差分
解码和记账不变量的 `cargo fuzz` target；运行 fuzz 需要 nightly toolchain。

## 延伸阅读

- [README](../README.zh_CN.md)
- [English guide](user_guide.md)
- [API 文档](https://docs.rs/qubit-budget)
