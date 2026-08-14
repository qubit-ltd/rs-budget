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

JSON value accounting is explicit and all-or-nothing. Stage every measurement
for one complete value, then publish it only after the enclosing operation has
succeeded:

```rust
use qubit_budget::json::JsonMeasurement;
use qubit_budget::json::JsonValueLimits;

let mut budget = JsonValueLimits::empty()
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

Each `try_admit` is atomic for its measurement. `commit` publishes all staged
value nodes and payload bytes; dropping the transaction, including during
unwinding, discards only its staged value accounting.

## JSON Atomicity

`JsonDecodeAttempt` and `JsonEncodeAttempt` separate irreversible I/O charges
from transactional value accounting. The matrix below states the contract that
high-level JSON integrations must preserve.

| Scenario | Input | Normalized input | Value | Output |
| --- | --- | --- | --- | --- |
| Strict decode succeeds | retained | not applicable | committed | not applicable |
| Strict decode fails | retained | not applicable | rolled back | not applicable |
| Lenient decode fails | retained | retained | rolled back | not applicable |
| Buffered `Vec<u8>` output fails | not applicable | not applicable | rolled back | success-only; no `Vec` means no output charge |
| Buffered writer partially fails | not applicable | not applicable | rolled back | each accepted prefix is retained immediately |
| Incremental writer fails | not applicable | not applicable | rolled back | each accepted prefix is retained immediately |
| One value in a stream fails | retained across values | retained across values | only the current value rolls back | previously accepted output remains retained |

Raw and normalized input are charged immediately when an attempt accepts them.
For an encoder that first produces a complete `Vec<u8>`, output is charged only
after that complete output succeeds. Writer-oriented encoders instead charge
each accepted prefix immediately; a later error does not undo bytes the writer
has already accepted.

The transaction is a budget boundary, not a general rollback mechanism.
Dropping it cannot undo writes already accepted by a writer, callback effects,
`Hasher` updates, or object mutation. A stream normally creates one independent
attempt per complete top-level value, while higher-level grouping may use one
transaction for a larger business operation. A handled rejection may continue
to use that transaction and commit the measurements that remain admissible.

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
- [User guide](doc/user_guide.md)
- [中文用户指南](doc/user_guide.zh_CN.md)
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
