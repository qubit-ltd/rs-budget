# qubit-budget

[![Rust CI](https://github.com/qubit-ltd/rs-budget/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-budget/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-budget/coverage-badge.json)](https://qubit-ltd.github.io/rs-budget/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-budget.svg?color=blue)](https://crates.io/crates/qubit-budget)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

`qubit-budget` gives Qubit Rust applications exact, finite resource limits and
accounting primitives. Use it when a parser, converter, serializer, or I/O
boundary must reject oversized work without relying on unchecked arithmetic or
format-specific policy.

## Installation

```toml
[dependencies]
qubit-budget = "0.4"
```

Enable `json`, `big-integer`, `big-decimal`, or `time` only when the associated
budget types are needed.

## Quick Start

Suppose an HTTP handler accepts a request body and must stop processing after
8 KiB. The handler can charge bytes as they arrive and inspect the remaining
capacity without maintaining a second counter:

```rust
use qubit_budget::ResourceBudget;

let mut body_budget = ResourceBudget::new("request body bytes", 8_u64);
body_budget.try_consume(3)?;
assert_eq!(body_budget.remaining(), 5);
# Ok::<(), qubit_budget::BudgetError<&str>>(())
```

`ResourceBudget::try_consume` is atomic: a request larger than the remaining
capacity returns a structured error and leaves the budget unchanged.

## What It Provides

| Feature | Adds |
| --- | --- |
| `json` | JSON resource identities, limits, and mutable decode/encode sessions |
| `big-integer` | `BigIntegerLimits` |
| `big-decimal` | `BigDecimalLimits` |
| `time` | Clock-backed `TimeBudget` |

The crate provides `ResourceLimit`, `ResourceBudget`, `ResourcePool`,
`StructureLimits`, `StructureBudget`, and string, numeric, duration, and time
helpers. A dimension that is not configured is represented by `Option`, not by
an unlimited budget object.

JSON limits and sessions live here so configuration, metadata, value objects,
and format adapters can share the same accounting contract. Parsing,
normalization, traversal, Serde adapters, and application error policies
remain in [`qubit-json`](https://crates.io/crates/qubit-json).

## Boundaries

This crate does not parse JSON, perform I/O, allocate output, select
application limits, or define application-specific error policies.

## Learn More

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
