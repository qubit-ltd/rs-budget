# qubit-budget

[![Rust CI](https://github.com/qubit-ltd/rs-budget/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-budget/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-budget/coverage-badge.json)](https://qubit-ltd.github.io/rs-budget/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-budget.svg?color=blue)](https://crates.io/crates/qubit-budget)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

`qubit-budget` helps Rust libraries and services put explicit, finite limits on
work: bytes read, nodes visited, output produced, open resources, or elapsed
time. It keeps the accounting separate from parsing and I/O, so callers can
reject oversized work with structured errors and predictable state changes.

## Installation

```toml
[dependencies]
qubit-budget = "0.3"
```

The crate has no default features. Enable an integration only when it is
needed:

```toml
[dependencies]
qubit-budget = { version = "0.3", features = ["json"] }
```

Available features: `json`, `big-integer`, `big-decimal` (which enables
`big-integer`), and `time`. The minimum supported Rust version is 1.94.

## Quick Start

Suppose one response may contain no more than eight bytes. Charge each accepted
chunk to one budget. A failed charge does not change that budget:

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

This is the basic pattern: choose a meaningful resource name, configure a
limit at the boundary you own, and let the caller decide how to handle a typed
error.

## Choose the right primitive

| Need | Type | What happens on success | What happens on failure |
| --- | --- | --- | --- |
| Check one value, such as nesting depth | `ResourceLimit` | No state changes | Reports the observed value and limit |
| Spend an allowance that cannot return | `ResourceBudget` | Reduces `remaining` | Budget is unchanged |
| Reuse capacity that callers return | `ResourcePool` | Acquire or release changes the pool | Pool is unchanged |

`ResourceBudget` is not cloneable: copying it would duplicate a finite
allowance. `ResourcePool` is only in-memory accounting; it does not wait,
synchronize access, issue RAII permits, or enforce fairness.

## What it provides

- Atomic charges for one budget and all-or-nothing charges for a group.
- Checked conversion of native `usize` and `u64` measurements.
- Structured errors for exceeded limits, insufficient budget, conversion, and
  invalid pool release.
- Reusable structure limits for depth, nodes, container sizes, and key bytes.
- Transactional string rendering: bytes are charged only after a complete,
  valid UTF-8 string is produced.
- Optional limits for JSON, strings, big integers, big decimals, durations,
  and clock-backed deadlines.

With `json`, an attempt distinguishes immediate I/O accounting from
transactional value accounting. Accepted input and writer output remain
charged; staged JSON value usage is published only by `commit`. The user guide
explains this boundary with a complete decode scenario.

## JSON transaction boundary

Use a transaction to stage measurements for one complete value. It publishes
them only after the surrounding operation succeeds:

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
transaction.commit();
# Ok::<(), qubit_budget::MeasuredBudgetError<qubit_budget::json::JsonResource, usize>>(())
```

The complete contract is intentionally asymmetric:

| Scenario | Input | Normalized input | Value | Output |
| --- | --- | --- | --- | --- |
| Strict decode succeeds | retained | not applicable | committed | not applicable |
| Strict decode fails | retained | not applicable | rolled back | not applicable |
| Lenient decode fails | retained | retained | rolled back | not applicable |
| Buffered `Vec<u8>` output fails | not applicable | not applicable | rolled back | success-only; no `Vec` means no output charge |
| Buffered writer partially fails | not applicable | not applicable | rolled back | each accepted prefix is retained immediately |
| Incremental writer fails | not applicable | not applicable | rolled back | each accepted prefix is retained immediately |
| One value in a stream fails | retained across values | retained across values | only the current value rolls back | previously accepted output remains retained |

Raw input and normalized input are charged immediately. Dropping a transaction
cannot undo an accepted prefix, callback effect, `Hasher` update, or object
mutation. A higher-level operation may select a wider transaction boundary, but
only `commit` publishes staged value accounting.

## What it does not do

This crate does not parse JSON, perform I/O, choose application limit values,
allocate permits, wait for pool capacity, or define recovery policy. For JSON
parsing and Serde integration, use an adapter such as
[`qubit-json`](https://crates.io/crates/qubit-json).

## Learn More

- [User guide](doc/user_guide.md): a scenario-led explanation of JSON
  accounting, errors, transactions, and troubleshooting.
- [API documentation](https://docs.rs/qubit-budget)

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
