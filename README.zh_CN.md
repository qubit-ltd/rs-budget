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
qubit-budget = "0.3"
```

默认 feature 集为空，按需启用扩展：

| Feature | 提供内容 |
| --- | --- |
| `json` | 通过 `JsonLimits` 和 `JsonBudget` 提供 JSON 输入/输出与结构限制 |
| `serde-json` | 提供带预算检查的 Serde JSON 序列化/反序列化 adapter，同时启用 `json` |
| `time` | 显式时长预算 `DurationBudget` 与单调时钟预算 `TimeBudget` |

最低支持的 Rust 版本为 1.94。

下面的 Serde JSON 端到端示例需要启用 `serde-json`，并由应用直接声明自己使用的
类型依赖：

```toml
[dependencies]
qubit-budget = { version = "0.3", features = ["serde-json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## 快速开始

假设 wire decoder 需要在输入变成无界的内存对象之前，拒绝超过深度、节点、容器、key
或字节策略的输入。启用 `serde-json` 后，crate 同时提供 parser adapter 和预算会话；
只启用 `json` 时，也可以把同一个会话交给其他 parser：

```rust
use qubit_budget::{JsonLimits, StructureLimits};
use qubit_budget::from_slice_with_budget;
use qubit_budget::to_vec_with_budget;
use serde::Deserialize;
use serde::Serialize;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
#[derive(Debug, Deserialize, Serialize)]
struct Document {
    items: Vec<String>,
}

let input = br#"{"items":["alpha"]}"#;

let structure_limits = StructureLimits::new()
    .with_max_depth(64)
    .with_max_nodes(100_000)
    .with_max_sequence_items(10_000)
    .with_max_map_entries(10_000)
    .with_max_key_bytes(256);
let limits = JsonLimits::new()
    .with_structure_limits(structure_limits)
    .with_max_input_bytes(1_048_576)
    .with_max_output_bytes(1_048_576)
    .with_max_string_bytes(256 * 1024)
    .with_max_number_bytes(4_096);
let mut budget = limits.budget();
let document: Document = from_slice_with_budget(input, &mut budget)?;
let output = to_vec_with_budget(&document, &mut budget)?;
assert_eq!(output, input);
# Ok(())
# }
```

adapter 会在返回解码结果前预检查完整输入字节、深度、累计节点、容器大小、对象 key
字节、字符串字节和数字字节；`to_writer_with_budget` 写出数据前会先检查完整输出及其
结构。若使用方自行驱动其他 parser，底层的 `enter_object`、`enter_array` 和
`enter_node` 仍可直接使用。下一份文档调用 `limits.budget()` 后会得到全新的会话，
不会继承上一份文档已经消耗的节点额度。

同样的状态规则也让失败处理保持明确。例如，节点上限为 1 时，第一次记账成功，
第二次记账失败，并且已接受的节点数不会被改变：

```rust
use qubit_budget::JsonLimits;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let mut budget = JsonLimits::new().with_max_nodes(1).budget();
budget.charge_node()?;
assert!(budget.charge_node().is_err());
# Ok(())
# }
```

decoder 可以匹配 `BudgetError`，再转换成自身已有的领域错误；`rs-value` 就是把
JSON 资源事实转换为 `ValueWireDecodeError`。

## 从配置到会话

`StructureLimits<R, Q>` 是可复用的结构部分，包含深度、累计节点、序列条目、映射
条目和结构化 key 字节限制。`JsonLimits<R, Q>` 在此基础上组合完整输入/输出字节、
字符串字节和数字表示字节限制。默认资源标识为 `StructureResource` 和 `JsonResource`；
如果下游需要自己的资源分类，也可以使用 `ResourceLimit<R, Q>` 提供自定义标识。

配置在预算会话中保持不可变。每次调用 `budget()` 都会创建拥有全新累计额度的会话。
未配置的维度用 `Option::None` 表示，而不是创建一个需要在每次调用中驱动的“无限”
预算对象。

## 下游如何使用

| 下游 crate | 实际用法 | 体现的价值 |
| --- | --- | --- |
| `rs-value` | 将 `StructureLimits` 组合进 `JsonLimits`，用一个 `JsonBudget` 执行 wire/JSON 遍历，再把 `BudgetError` 映射为 `ValueWireDecodeError`。 | 多维限制可以复用，不需要复制遍历记账逻辑或领域错误。 |
| `rs-redact` | 在嵌套诊断渲染器之间共享一次操作的输入、输出和掩码预算。 | 子组件不能静默重置外层操作的额度。 |
| `rs-http`、`rs-io`、`rs-fs` | 数据到达时为 response body、stream 和文件读取记账，包括长度未知的输入。 | 不同分块方式和 transport 错误不会破坏累计上限。 |
| `rs-retry` | 组合尝试次数 `ResourceBudget`、显式时长 `DurationBudget` 和连续 elapsed deadline。 | 同一个领域策略可以组合不同资源语义。 |

## 核心能力

| 需求 | 公开 API |
| --- | --- |
| 单次、包含边界的点检查 | `ResourceLimit<R, Q>` |
| 不可归还的累计消耗 | `ResourceBudget<R, Q>` |
| 可获取和释放的容量 | `ResourcePool<R, Q>` |
| 结构化失败事实 | `BudgetError<R, Q>`：`LimitExceeded`、`Insufficient`、`InvalidRelease` |
| 通用嵌套数据限制 | `StructureLimits<R, Q>`、`StructureBudget<R, Q>` |
| JSON 输入/输出和遍历限制（`json`） | `JsonLimits<R, Q>`、`JsonBudget<R, Q>`、`JsonResource` |
| 显式或基于时钟的时间限制（`time`） | `DurationBudget<R>`、`TimeBudget<R, C>` |

数量使用精确的无符号类型。通用资源预算默认为 `u64`；结构化和 JSON 辅助类型默认
使用 `usize`，因为它们通常记录长度、数量和深度。

## 边界

- 核心记账 API 不解析 JSON、不执行 I/O、不分配输出，也不决定使用方何时检查观测值。
  可选的 `serde-json` feature 提供带预算检查的 Serde JSON adapter，但仍不替使用方
  选择限制或领域错误策略。
- 它不选择默认上限、重试策略、脱敏策略、调度方式或应用专属错误类型。
- `BudgetError` 描述的是机制事实，不是所有应用都必须暴露的统一领域错误。使用方
  应在自己的公开边界根据资源和值的变体完成转换。
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
