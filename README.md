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
qubit-budget = "0.4"
```

The crate has no default features. Enable an integration only when it is
needed:

```toml
[dependencies]
qubit-budget = { version = "0.4", features = ["json"] }
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

## What it does not do

This crate does not parse JSON, perform I/O, choose application limit values,
allocate permits, wait for pool capacity, or define recovery policy. For JSON
parsing and Serde integration, use an adapter such as
[`qubit-json`](https://crates.io/crates/qubit-json).

## Learn More

- [English user guide](doc/user_guide.md): a scenario-led explanation of JSON
  accounting, errors, transactions, and troubleshooting.
- [中文用户手册](doc/user_guide.zh_CN.md)
- [API documentation](https://docs.rs/qubit-budget)
- [中文 README](README.zh_CN.md)

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
