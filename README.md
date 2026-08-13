# qubit-budget

[![Rust CI](https://github.com/qubit-ltd/rs-budget/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-budget/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-budget/coverage-badge.json)](https://qubit-ltd.github.io/rs-budget/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-budget.svg?color=blue)](https://crates.io/crates/qubit-budget)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

`qubit-budget` provides dependency-light, format-agnostic finite resource
limits and accounting primitives for Qubit Rust crates.

It owns generic `ResourceLimit`, `ResourceBudget`, `ResourcePool`,
`StructureLimits`, `StructureBudget`, and string, numeric, duration, and time
helpers. A budget is always finite; use `Option<ResourceBudget<_, _>>` for an
unconfigured dimension.

## Features

| Feature | Adds |
| --- | --- |
| `json` | JSON resource limits and budget sessions |
| `big-integer` | `BigIntegerLimits` |
| `big-decimal` | `BigDecimalLimits` |
| `time` | clock-backed `TimeBudget` |

The minimum supported Rust version is 1.94.

## Quick start

```rust
use qubit_budget::ResourceBudget;

let mut budget = ResourceBudget::new("body bytes", 8_u64);
budget.try_consume(3)?;
assert_eq!(budget.remaining(), 5);
# Ok::<(), qubit_budget::BudgetError<&str>>(())
```

`ResourceBudget` can be cloned to create an independent accounting snapshot.
Each snapshot remains valid and can be charged independently.

## JSON support

With the optional `json` feature, this crate owns JSON resource identities,
limits, and mutable sessions. Enable it with `qubit-budget = { version =
"0.4", features = ["json"] }` and import `Json*` budget types from
`qubit_budget::json`. Parsing, normalization, traversal, and Serde adapters
remain in [`qubit-json`](https://crates.io/crates/qubit-json).

```rust
use qubit_budget::json::JsonDecodeLimits;

let limits = JsonDecodeLimits::empty()
    .with_max_input_bytes(1024)
    .with_max_nodes(128)
    .with_max_string_bytes(4096);
assert_eq!(limits.max_input_bytes(), Some(1024));
```

## Boundaries

This crate does not parse JSON, perform I/O, allocate output, select limits,
or define application-specific error policies.

## Testing

Run `cargo test --all-features` for the complete feature set.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE).

## Contributing

Run the repository's `style-check.sh`, Clippy, tests, and documentation checks
before submitting a change.

## Author

Haixing Hu
