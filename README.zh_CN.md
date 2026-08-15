# qubit-budget

[![Rust CI](https://github.com/qubit-ltd/rs-budget/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-budget/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-budget/coverage-badge.json)](https://qubit-ltd.github.io/rs-budget/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-budget.svg?color=blue)](https://crates.io/crates/qubit-budget)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

`qubit-budget` 为 Rust 程序提供轻量的有限资源约束原语。解析器、序列化器、
转换器或 I/O 边界可以用它精确记账，在工作量超限时获得结构化错误，并明确知道
失败操作是否改变了状态。

## 安装

```toml
[dependencies]
qubit-budget = "0.4"
```

本 crate 默认不启用任何 feature。只需按实际用途选择集成能力：

```toml
[dependencies]
qubit-budget = { version = "0.4", features = ["json"] }
```

可用 feature 包括 `json`、`big-integer`、`big-decimal` 和 `time`；其中
`big-decimal` 会同时启用 `big-integer`。项目支持的最低 Rust 版本为 1.94。

## 快速开始

假设服务在构造一次响应时最多允许产生 8 字节。成功计费会扣减余额；请求超出
余额时，预算保持不变，错误中包含完整的诊断信息：

```rust
use qubit_budget::ResourceBudget;

let mut response = ResourceBudget::new("response-bytes", 8_u64);
response.try_consume(5).expect("the first chunk fits");

let error = response
    .try_consume(4)
    .expect_err("only three bytes remain");

assert_eq!(error.resource(), &"response-bytes");
assert_eq!(error.limit(), 8);
assert_eq!(error.remaining(), 3);
assert_eq!(error.requested(), 4);
assert_eq!(response.used(), 5);
assert_eq!(response.remaining(), 3);
```

点上限、可释放资源池、结构限制、原生值测量和 JSON 记账都沿用同样的资源标识与
结构化诊断方式。

## 为什么需要这个项目

资源保护逻辑经常散落在临时计数器、未经检查的整数转换和格式专属错误中。出错后，
调用方很难判断状态是否已经改变，也难以定位究竟是哪项限制拒绝了操作。

`qubit-budget` 把三种常见策略明确区分开：

| 策略 | 类型 | 状态模型 |
| --- | --- | --- |
| 校验一次测量值 | `ResourceLimit` | 不可变的包含式上限 |
| 消耗一份有限额度 | `ResourceBudget` | 单调递减，不可释放 |
| 借用可复用容量 | `ResourcePool` | 显式获取与释放 |

所有资源量都使用精确的无符号整数。未配置的维度由 `Option::None` 表示，不会在
内部创建一个含义模糊的“无限预算”。

## 核心能力

- 单项预算原子计费，以及多项预算的全有或全无计费。
- 对原生 `usize` 与 `u64` 测量执行无截断的检查转换。
- 提供点上限、预算不足、预算组、数量转换和非法释放等结构化错误。
- 复用深度、节点数、容器大小和键字节数等结构限制。
- 事务式生成 UTF-8 字符串，仅在完整成功后提交字节计费。
- 可选支持字符串、大整数、大十进制数、时长及基于时钟的截止时间。
- 启用 `json` 后，可配置与方向无关的 value 限制，并通过 decode/encode session
  区分立即生效的 I/O 计费和可回滚的 value 记账。

在 JSON 流程中，丢弃 attempt 只会回滚尚未提交的 value 记账。decoder 已接受的
原始或规范化输入，以及 writer 已接受的输出前缀，仍然保留计费。完整原子性模型和
集成方式见用户手册。

## JSON transaction 边界

启用 `json` feature 后，先暂存一个完整 value 的全部测量，只有外围操作成功后才
发布：

```rust
use qubit_budget::json::JsonMeasurement;
use qubit_budget::json::JsonValueLimits;

let mut budget = JsonValueLimits::<JsonResource, usize>::new()
    .with_max_nodes(8)
    .with_max_string_bytes(16)
    .budget();
let mut transaction = budget.transaction();
transaction.try_admit(JsonMeasurement::String {
    depth: 1,
    bytes: 5,
})?;
transaction.commit();
# Ok::<(), qubit_budget::MeasuredBudgetError<qubit_budget::json::JsonResource, usize>>(())
```

完整的 attempt 合同如下：

| 场景 | input | normalized input | value | output |
| --- | --- | --- | --- | --- |
| strict decode 成功 | 保留 | 不适用 | 提交 | 不适用 |
| strict decode 失败 | 保留 | 不适用 | 回滚 | 不适用 |
| lenient decode 失败 | 保留 | 保留 | 回滚 | 不适用 |
| 缓冲的 `Vec<u8>` output 失败 | 不适用 | 不适用 | 回滚 | 只在成功时计费；没有 `Vec` 就不计 output |
| buffered writer 部分失败 | 不适用 | 不适用 | 回滚 | 每个 accepted prefix 立即保留 |
| incremental writer 失败 | 不适用 | 不适用 | 回滚 | 每个 accepted prefix 立即保留 |
| stream 中单个 value 失败 | 跨 value 持续累计 | 跨 value 持续累计 | 仅当前 value 回滚 | 之前接受的 output 继续保留 |

attempt 一旦接受 raw input 或 normalized input，就会立即计费。丢弃 transaction
无法撤销 accepted prefix、callback 副作用、`Hasher` 更新或对象 mutation。
higher-level 操作可以有意扩大 transaction 边界，但暂存的 value 记账仍然只通过
`commit` 发布。

## 使用边界

本 crate 不负责解析 JSON、执行 I/O、分配 permit、等待资源池容量、替应用选择
限制值，也不定义应用自己的恢复策略。`ResourcePool` 只是内存中的记账原语，并非
同步 semaphore 或 RAII permit 系统。JSON 解析、规范化、遍历和 Serde 集成应由
[`qubit-json`](https://crates.io/crates/qubit-json) 等格式适配层实现。

## 延伸阅读

- [中文用户手册](doc/user_guide.zh_CN.md)
- [English user guide](doc/user_guide.md)
- [API 文档](https://docs.rs/qubit-budget)
- [English README](README.md)
- [项目仓库](https://github.com/qubit-ltd/rs-budget)

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
Pull Request 前运行 `./align-ci.sh`格式化代码，运行`./ci-check.sh`对齐CI要求。

## 作者

**Haixing Hu** - *Qubit Co. Ltd.*

仓库地址：[https://github.com/qubit-ltd/rs-budget](https://github.com/qubit-ltd/rs-budget)
