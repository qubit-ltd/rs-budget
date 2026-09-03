# qubit-budget

[![Rust CI](https://github.com/qubit-ltd/rs-budget/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-budget/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-budget/coverage-badge.json)](https://qubit-ltd.github.io/rs-budget/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-budget.svg?color=blue)](https://crates.io/crates/qubit-budget)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

`qubit-budget` 用于给 Rust 库和服务中的工作量设定明确、有限的上限：例如读取的
字节数、遍历的节点数、已写出的内容、已占用的资源或耗用时间。它只负责记账，不把
策略绑定到某个解析器或 I/O 实现，因此超限时可以得到结构化错误，并能明确知道状态
是否变化。

## 安装

```toml
[dependencies]
qubit-budget = "0.5"
```

本 crate 默认不启用 feature。只有需要集成能力时才按需开启：

```toml
[dependencies]
qubit-budget = { version = "0.5", features = ["json"] }
```

可选 feature 为 `json`、`big-integer`、`big-decimal`（会同时启用
`big-integer`）和 `time`。最低支持 Rust 1.94。

## 快速开始

假设一次响应最多允许写出 8 字节。每接受一段内容，就向同一份预算记账；如果本次
请求超出余额，预算不会变化：

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
```

实际使用时，为资源取一个稳定且有含义的名称，在你负责的边界配置上限；出现类型化
错误后，由调用方决定拒绝、重试还是采取其他恢复措施。

## 先选对原语

| 需求 | 类型 | 成功后 | 失败后 |
| --- | --- | --- | --- |
| 校验一次独立测量，例如嵌套深度 | `ResourceLimit` | 不修改状态 | 返回测量值和上限 |
| 消耗不可归还的额度 | `ResourceBudget` | 扣减 `remaining` | 预算保持不变 |
| 使用后会显式归还的容量 | `ResourcePool` | 获取或释放会改变资源池 | 资源池保持不变 |
| 将可复用容量绑定到所有权生命周期 | `ManagedResourcePool` | 返回 RAII permit | 资源池保持不变 |

`ResourceBudget` 不实现 `Clone`，避免一份有限额度被复制成两份。
`ResourcePool` 只是内存中的手工记账对象：它不等待、不提供同步，也不保证公平性。
`ManagedResourcePool` 是可克隆的同步句柄，permit 会在 Drop 时归还容量；它同样不等待、
不保证公平性。

## 核心能力

- 单项预算的原子计费，以及预算组的全有或全无计费。
- 对原生 `usize` 和 `u64` 测量进行精确、无截断的转换检查。
- 为超出上限、预算不足、数量转换和资源池错误提供结构化错误。
- 可复用的结构限制：深度、节点数、容器大小和对象键字节数。
- 事务式字符串生成：只有完整生成有效 UTF-8 字符串后才扣减字节预算。
- 可选的 JSON、字符串、大整数、大十进制数、时长和基于时钟的截止时间限制。

启用 `json` 后，一次处理会区分“立即入账的 I/O”与“可回滚的 value 用量”：已接受的
输入和 writer 输出仍会保留；暂存的 JSON value 用量只有调用 `commit` 才会生效。完整
过程见用户手册中的 JSON 解码场景。

## JSON transaction 边界

transaction 会暂存一个完整 value 的测量结果，只有外围操作成功才正式发布：

```rust
use qubit_budget::json::JsonMeasurement;
use qubit_budget::json::JsonResource;
use qubit_budget::json::JsonValueLimits;

let mut budget = JsonValueLimits::<JsonResource, usize>::builder()
    .max_nodes(8)
    .max_string_bytes(16)
    .build()
    .budget();
let mut transaction = budget.transaction();
transaction.try_admit(JsonMeasurement::String {
    depth: 1,
    bytes: 5,
})?;
transaction.commit()?;
# Ok::<(), qubit_budget::MeasuredBudgetError<qubit_budget::json::JsonResource, usize>>(())
```

raw input 和 normalized input 仍然立即入账；transaction 只暂存 value 用量，因此应在
完整 value 成功后调用 `commit`。value admission 被拒绝会使该 transaction 进入 poisoned
状态并禁止发布，但已经接受的 I/O 和 output 仍然保留计费。完整原子性矩阵和恢复边界请看
用户手册与设计文档。

## 不负责什么

本 crate 不解析 JSON，不执行 I/O，不替应用选择限额，不等待资源池
容量，也不定义错误后的恢复策略。JSON 解析和 Serde 集成应交给
[`qubit-json`](https://crates.io/crates/qubit-json) 等适配层。

## 延伸阅读

- [用户手册](doc/user_guide.zh_CN.md)：通过完整 JSON 场景说明记账、事务、错误和排障。
- [设计文档](doc/design.zh_CN.md)：说明不变量、状态迁移和 feature 边界。
- [API 文档](https://docs.rs/qubit-budget)

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
