# rs-budget

[![Rust CI](https://github.com/qubit-ltd/rs-budget/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-budget/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-budget/coverage-badge.json)](https://qubit-ltd.github.io/rs-budget/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-budget.svg?color=blue)](https://crates.io/crates/qubit-budget)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

Dependency-light resource limit and budget accounting primitives for Qubit Rust crates.

## Intended Users

Rust library authors who need to enforce bounded input, output, depth, node, or
item work while keeping domain-specific policies and diagnostics in their own
crates.

## Installation

Add the published crate to your `Cargo.toml`:

```toml
[dependencies]
# Default feature set.
qubit-budget = "0.3"

# Or enable duration and monotonic-deadline budgets when needed.
# qubit-budget = { version = "0.3", features = ["time"] }
```

## Quick Start

```rust
use qubit_budget::{ResourceBudget, ResourceLimit};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Resource {
    Nodes,
}

let mut budget = ResourceBudget::new(Resource::Nodes, ResourceLimit::new(100));
budget.try_consume(40).expect("within the budget");
assert_eq!(budget.remaining(), 60);
```

An unconfigured dimension is represented by the caller as `None`; the crate
does not construct unlimited or no-op budget objects:

```rust
let budget: Option<ResourceBudget<Resource>> = None;
```

## Capabilities

- immutable finite single-dimension limits through `ResourceLimit`;
- monotonic, non-releasable `ResourceBudget<R>` with `u64` quantities;
- failure-atomic, reusable `ResourcePool<R>` with one unified error type;
- optional `time` feature with explicit `DurationBudget<R>` and continuous
  `TimeBudget<R, C>`.

## Limitations

The crate intentionally does not define JSON, Serde, I/O, redaction, parser,
default-limit, or domain-error policy. Consuming crates should retain their own
public resource types and translate the structured errors into their established
errors. `DurationBudget` consumes only durations explicitly submitted by the
caller; `TimeBudget` includes operation, waiting and backoff time.

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
