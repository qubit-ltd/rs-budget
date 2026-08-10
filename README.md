# rs-budget

[![Rust CI](https://github.com/qubit-ltd/rs-budget/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-budget/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-budget/coverage-badge.json)](https://qubit-ltd.github.io/rs-budget/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-budget.svg?color=blue)](https://crates.io/crates/qubit-budget)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

`qubit-budget` provides dependency-light accounting primitives for Rust libraries
that must bound input, traversal, output, or elapsed-time resources. A consumer
chooses the resource identity, limit, and error mapping, so a transport, parser,
filesystem, or redaction crate can enforce finite work without inheriting a
shared domain policy.

## Installation

```toml
[dependencies]
qubit-budget = "0.3"
```

The default feature set is empty. Enable only the extensions you need:

| Feature | Adds |
| --- | --- |
| `json` | `JsonLimits` and `JsonBudget` for JSON measurements and node accounting |
| `time` | `DurationBudget` and clock-backed `TimeBudget` |

The minimum supported Rust version is 1.94.

## Quick Start

Suppose an HTTP or I/O adapter receives a body in chunks. It can charge each
accepted chunk before buffering it, while retaining its own response or stream
error type at the boundary:

```rust
use qubit_budget::ResourceBudget;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let chunks = [b"hello".as_slice(), b" world".as_slice()];
let mut body_budget = ResourceBudget::new("response body", 11_usize);
let mut body = Vec::new();

for chunk in chunks {
    body_budget.try_consume(chunk.len())?;
    body.extend_from_slice(chunk);
}

assert_eq!(body, b"hello world");
# Ok(())
# }
```

`try_consume` is atomic: an oversized request returns
`BudgetError::Insufficient` and leaves the remaining balance unchanged. The
same pattern is used by downstream Qubit crates such as `rs-http`, `rs-io`,
`rs-fs`, and `rs-redact` for bounded bodies, streams, files, and diagnostics.

## Why This Project Exists

Resource limits are usually owned by the crate that understands the input or
operation. Reimplementing finite accounting in every adapter leads to subtly
different failure and state-transition rules. This crate centralizes those
mechanics while leaving resource names, defaults, scheduling, and public error
policy with the consuming crate.

## What It Provides

| Need | Public API |
| --- | --- |
| One inclusive point check | `ResourceLimit<R, Q>` |
| Non-releasable cumulative consumption | `ResourceBudget<R, Q>` |
| Releasable capacity with acquisition and release | `ResourcePool<R, Q>` |
| Structured failures | `BudgetError<R, Q>`: `LimitExceeded`, `Insufficient`, `InvalidRelease` |
| Generic nested-data limits | `StructureLimits` and `StructureBudget` |
| JSON input and traversal limits (`json`) | `JsonLimits`, `JsonBudget`, and `JsonResource` |
| Explicit durations or continuous deadlines (`time`) | `DurationBudget<R>` and `TimeBudget<R, C>` |

Quantities are exact unsigned values and default to `u64`; structural and JSON
helpers use `usize`. An unconfigured dimension is represented by `Option::None`,
not by an unlimited budget object. New budget sessions start with their full
configured capacity.

## Boundaries and Guarantees

- Point limits compare one observed value with an inclusive maximum and do not
  accumulate across calls.
- Resource budgets consume capacity monotonically; failed requests do not
  change state. Pools can release capacity explicitly, but do not provide
  synchronization, waiting, fairness, permits, or cancellation.
- `StructureBudget` and `JsonBudget` do not parse input. The consuming parser or
  traversal decides what to measure and when to check or charge it.
- `DurationBudget` counts only durations explicitly submitted by the caller.
  `TimeBudget` samples an injected monotonic clock and therefore includes
  operation, waiting, queueing, and backoff time.
- The crate does not provide I/O, Serde integration, default limits, retry
  policy, redaction, or application-specific error types.

## Learn More

- [English user guide](doc/user_guide.md)
- [中文用户指南](doc/user_guide.zh_CN.md)
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
