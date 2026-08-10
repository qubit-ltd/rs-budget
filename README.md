# rs-budget

[![Rust CI](https://github.com/qubit-ltd/rs-budget/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-budget/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-budget/coverage-badge.json)](https://qubit-ltd.github.io/rs-budget/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-budget.svg?color=blue)](https://crates.io/crates/qubit-budget)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

Dependency-light finite resource limits and budget accounting primitives for
Qubit Rust crates. They let a consuming crate enforce its own input and
processing limits without adopting this crate's domain policy or public errors.

## Intended Users

Rust library authors who need bounded input, depth, node, item, or byte work,
while retaining their own resource taxonomy, defaults, diagnostics, and error
types.

## Installation

Add the published crate to your `Cargo.toml`:

```toml
[dependencies]
# Default feature set.
qubit-budget = "0.3"

# Enable JSON limit accounting only when it is needed.
# qubit-budget = { version = "0.3", features = ["json"] }

# Or enable duration and monotonic-deadline budgets when needed.
# qubit-budget = { version = "0.3", features = ["time"] }
```

## Quick Start

A decoder that already owns JSON traversal can configure its limits and charge
the traversal itself. The `json` feature supplies no Serde integration and no
parser; it only provides the limit configuration and accounting APIs.

```rust
use qubit_budget::JsonLimits;

let limits = JsonLimits::new()
    .with_max_depth(64)
    .with_max_nodes(100_000);
let mut budget = limits.budget();
budget.check_depth(1)?;
budget.charge_node()?;
```

`check_depth`, input-byte, array-item, map-entry, string-byte, and number-byte
operations are point checks: each independently compares one observed value
with an inclusive maximum. `charge_node` is different: every call consumes one
unit from the session's cumulative node budget. A fresh `limits.budget()` call
starts a fresh node session.

## Capabilities

- `ResourceLimit<R, Q>` checks one inclusive point measurement.
- `ResourceBudget<R, Q>` records finite, non-releasable cumulative use; and
  `ResourcePool<R, Q>` records finite releasable capacity.
- `BudgetError<R, Q>` is the unified structured failure type:
  `LimitExceeded` for point checks, `Insufficient` for budget or acquisition
  failures, and `InvalidRelease` for releasing more than is in use.
- `StructureLimits` creates `StructureBudget` sessions for depth, cumulative
  nodes, sequence items, and map entries without imposing a data format.
- The optional `json` feature adds `JsonLimits` and `JsonBudget` for JSON input
  bytes (the complete input), root-inclusive depth, cumulative nodes,
  array/object sizes, decoded UTF-8 string bytes, and numeric lexical bytes.
- The optional `time` feature provides explicit `DurationBudget<R>` and
  continuous `TimeBudget<R, C>`.

## Error Boundaries

`BudgetError` reports facts, not application policy. Consumers should match its
resource and variant, then map it to their own public domain error at their
boundary. The same type covers point, cumulative-budget, and pool failures.

## Limitations

The crate intentionally does not define a JSON parser, Serde integration, I/O,
redaction, default limits, retry policy, or domain-error policy. A parser or
wire crate decides when to call the checks and charges, and translates
`BudgetError` into its established error model. `DurationBudget` consumes only
durations explicitly submitted by the caller; `TimeBudget` includes operation,
waiting, queueing, and backoff time.

## Learn More

Read the [English user guide](doc/user_guide.md),
[中文用户指南](doc/user_guide.zh_CN.md), or the [API documentation](https://docs.rs/qubit-budget).

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
