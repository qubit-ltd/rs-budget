# qubit-budget

[![Rust CI](https://github.com/qubit-ltd/rs-budget/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-budget/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-budget/coverage-badge.json)](https://qubit-ltd.github.io/rs-budget/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-budget.svg?color=blue)](https://crates.io/crates/qubit-budget)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

`qubit-budget` provides dependency-light primitives for enforcing finite
resource limits in Rust. It helps parsers, serializers, converters, and I/O
boundaries reject oversized work with exact accounting, structured errors, and
well-defined mutation semantics.

## Installation

```toml
[dependencies]
qubit-budget = "0.4"
```

The crate has no default features. Enable only the integrations you need:

```toml
[dependencies]
qubit-budget = { version = "0.4", features = ["json"] }
```

Available features are `json`, `big-integer`, `big-decimal`, and `time`.
`big-decimal` also enables `big-integer`. The minimum supported Rust version is
1.94.

## Quick Start

Suppose a service may spend at most 8 bytes while constructing one response.
Successful charges reduce the remaining allowance; a rejected charge leaves
the budget unchanged and reports the exact failure facts:

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

The same resource identity and structured diagnostics carry through point
limits, releasable pools, structural limits, measured values, and JSON
accounting.

## Why This Project Exists

Resource protection is often scattered across ad hoc counters, unchecked
integer conversions, and format-specific error types. That makes it difficult
to answer whether a failed operation changed state or which limit rejected it.

`qubit-budget` separates three common policies:

| Policy | Type | State model |
| --- | --- | --- |
| Validate one observation | `ResourceLimit` | Immutable inclusive maximum |
| Spend a finite allowance | `ResourceBudget` | Monotonic, non-releasable consumption |
| Borrow reusable capacity | `ResourcePool` | Explicit acquire and release |

All quantities are exact unsigned integers. An unconfigured dimension is
represented by `Option::None`; the crate does not create a hidden “unlimited”
budget.

## What It Provides

- Atomic single-budget charges and all-or-nothing grouped charges.
- Checked conversion from native `usize` and `u64` measurements.
- Structured point-limit, insufficient-budget, grouped-budget, conversion, and
  invalid-release errors.
- Reusable structural limits for depth, nodes, container sizes, and key bytes.
- Transactional UTF-8 string rendering that commits bytes only after success.
- Optional limits for strings, big integers, big decimals, durations, and
  clock-backed deadlines.
- With `json`, direction-independent value limits plus decode and encode
  sessions that distinguish immediate I/O charges from transactional value
  accounting.

For JSON, dropping an attempt rolls back only staged value accounting. Raw or
normalized input already accepted by a decoder, and output prefixes already
accepted by a writer, remain charged. See the user guide for the complete
atomicity model and integration patterns.

## JSON Transaction Boundary

With the `json` feature, stage all measurements for one complete value and
publish them only after the enclosing operation succeeds:

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

The complete attempt contract is:

| Scenario | Input | Normalized input | Value | Output |
| --- | --- | --- | --- | --- |
| Strict decode succeeds | retained | not applicable | committed | not applicable |
| Strict decode fails | retained | not applicable | rolled back | not applicable |
| Lenient decode fails | retained | retained | rolled back | not applicable |
| Buffered `Vec<u8>` output fails | not applicable | not applicable | rolled back | success-only; no `Vec` means no output charge |
| Buffered writer partially fails | not applicable | not applicable | rolled back | each accepted prefix is retained immediately |
| Incremental writer fails | not applicable | not applicable | rolled back | each accepted prefix is retained immediately |
| One value in a stream fails | retained across values | retained across values | only the current value rolls back | previously accepted output remains retained |

Raw input and normalized input are charged as soon as an attempt accepts them.
Dropping a transaction cannot undo an accepted prefix, callback effect,
`Hasher` update, or object mutation. A higher-level operation may deliberately
choose a wider transaction boundary, but only `commit` publishes staged value
accounting.

## Boundaries

This crate does not parse JSON, perform I/O, allocate permits, wait for pool
capacity, choose application limits, or define application-specific recovery
policy. `ResourcePool` is an in-memory accounting primitive, not a synchronized
semaphore or RAII permit system. JSON parsing, normalization, traversal, and
Serde integration belong to format adapters such as
[`qubit-json`](https://crates.io/crates/qubit-json).

## Learn More

- [English user guide](doc/user_guide.md)
- [中文用户手册](doc/user_guide.zh_CN.md)
- [API documentation](https://docs.rs/qubit-budget)
- [中文 README](README.zh_CN.md)
- [Repository](https://github.com/qubit-ltd/rs-budget)

## Testing

```bash
# Run tests with the default feature set
cargo test

# Run tests with all declared features
cargo test --all-features

# Project CI checks
./ci-check.sh

# Check code coverage
./coverage.sh
```

## License

Copyright (c) 2025 - 2026. Haixing Hu. All rights reserved.

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for the
full license text.

## Contributing

Contributions are welcome. Please follow the Rust API guidelines, keep public
API documentation and tests current, and run `./align-ci.sh` to format code and
`./ci-check.sh` to satisfy CI requirements before submitting a pull request.

## Author

**Haixing Hu** - *Qubit Co. Ltd.*

Repository: [https://github.com/qubit-ltd/rs-budget](https://github.com/qubit-ltd/rs-budget)
