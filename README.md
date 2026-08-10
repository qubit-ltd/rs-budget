# rs-budget

[![Rust CI](https://github.com/qubit-ltd/rs-budget/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-budget/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-budget/coverage-badge.json)](https://qubit-ltd.github.io/rs-budget/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-budget.svg?color=blue)](https://crates.io/crates/qubit-budget)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

`qubit-budget` gives Rust libraries composable resource constraints for
untrusted or potentially unbounded work. It separates the accounting mechanics
from the policy owned by a parser, wire protocol, transport, filesystem, retry
executor, or diagnostic renderer: the consumer chooses resource identities,
defaults, and public errors, while `qubit-budget` keeps each operation's limits
and state transitions consistent.

## Why Resource Limits Must Compose

For structured input, a maximum input size is only one dimension of the
problem. A small document can still be deeply recursive, contain many tiny
nodes, fan out through a large object or array, carry a very large key, or
expand into a larger output. These observations have different meanings and
must not silently share one counter.

A correct boundary therefore needs both point limits and cumulative budgets:

- Point checks compare one observed depth, container size, key length, string
  length, or numeric representation with an inclusive maximum.
- Cumulative budgets charge work across the whole operation, so a recursive
  child or nested adapter cannot reset the parent's allowance.
- A combined operation checks all point limits before consuming its node
  allowance. A failed charge leaves the cumulative balance unchanged.
- The accounting layer reports exact resource facts; the consuming crate maps
  them to its own error model and decides when each observation is made.

## Installation

```toml
[dependencies]
qubit-budget = "0.3"
```

The default feature set is empty. Enable only the extensions you need:

| Feature | Adds |
| --- | --- |
| `json` | Generic JSON input/output and structural limits through `JsonLimits` and `JsonBudget` |
| `serde-json` | Budget-aware Serde JSON deserialization and serialization adapters; also enables `json` |
| `time` | Explicit-duration `DurationBudget` and monotonic-clock `TimeBudget` |

The minimum supported Rust version is 1.94.

For the end-to-end Serde JSON example below, enable `serde-json` and declare
the types used by your application directly:

```toml
[dependencies]
qubit-budget = { version = "0.3", features = ["serde-json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## Quick Start

Imagine a wire decoder that must reject inputs that exceed its depth, node,
container, key, or byte policy before they become an unbounded in-memory value.
With `serde-json`, the crate supplies the parser adapter as well as the budget
session; with `json` alone, the same session can be driven by another parser.

```rust
use qubit_budget::{JsonLimits, StructureLimits};
use qubit_budget::from_slice_with_budget;
use qubit_budget::to_vec_with_budget;
use serde::Deserialize;
use serde::Serialize;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
#[derive(Debug, Deserialize, Serialize)]
struct Document {
    items: Vec<String>,
}

let input = br#"{"items":["alpha"]}"#;

let structure_limits = StructureLimits::new()
    .with_max_depth(64)
    .with_max_nodes(100_000)
    .with_max_sequence_items(10_000)
    .with_max_map_entries(10_000)
    .with_max_key_bytes(256);
let limits = JsonLimits::new()
    .with_structure_limits(structure_limits)
    .with_max_input_bytes(1_048_576)
    .with_max_output_bytes(1_048_576)
    .with_max_string_bytes(256 * 1024)
    .with_max_number_bytes(4_096);
let mut budget = limits.budget();
let document: Document = from_slice_with_budget(input, &mut budget)?;
let output = to_vec_with_budget(&document, &mut budget)?;
assert_eq!(output, input);
# Ok(())
# }
```

The adapter preflights complete input bytes, depth, cumulative nodes, container
sizes, object-key bytes, string bytes, and number bytes before returning the
decoded value. It checks the complete output size and its structure before
`to_writer_with_budget` writes anything. `enter_object`, `enter_array`, and
`enter_node` are the lower-level operations used when a consumer drives another
parser. Calling `limits.budget()` for another document creates a fresh session;
it does not share node usage with the previous document.

The same state rules make failure handling predictable. For example, a node
budget of one accepts its first charge and rejects the second without changing
the accepted count:

```rust
use qubit_budget::JsonLimits;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let mut budget = JsonLimits::new().with_max_nodes(1).budget();
budget.charge_node()?;
assert!(budget.charge_node().is_err());
# Ok(())
# }
```

The decoder can match `BudgetError` and translate it into its established
domain error, just as `rs-value` translates JSON resource facts into
`ValueWireDecodeError`.

## From Configuration to Session

`StructureLimits<R, Q>` is the reusable structural part: depth, cumulative
nodes, sequence items, map entries, and structural key bytes. `JsonLimits<R, Q>`
composes it with complete input/output bytes, string bytes, and numeric
representation bytes. The default resource identities are `StructureResource`
and `JsonResource`; custom resource identities can be supplied with
`ResourceLimit<R, Q>` when a consuming crate needs its own taxonomy.

The configuration is immutable from the accounting session's point of view.
Each `budget()` call creates a new session with fresh cumulative capacity. An
unconfigured dimension is represented by `Option::None`; it is not an
unlimited budget object that needs to be driven through every call.

## How Downstream Crates Use It

| Downstream crate | Actual use | What it demonstrates |
| --- | --- | --- |
| `rs-value` | Composes `StructureLimits` into `JsonLimits`, runs one `JsonBudget` through wire/JSON traversal, and maps `BudgetError` to `ValueWireDecodeError`. | Multi-dimensional limits can be reused without duplicating traversal accounting or domain errors. |
| `rs-redact` | Shares operation-scoped input, output, and mask budgets across nested diagnostic renderers. | A child component cannot silently reset the allowance for the enclosing operation. |
| `rs-http`, `rs-io`, `rs-fs` | Charges response bodies, streams, and file reads as data arrives, including unknown-length inputs. | The same cumulative invariant works independently of chunk boundaries and transport errors. |
| `rs-retry` | Combines an attempt-count `ResourceBudget`, an explicit `DurationBudget`, and a continuous elapsed-time deadline. | Different resources can use different accounting semantics in one domain policy. |

## What It Provides

| Need | Public API |
| --- | --- |
| One inclusive point check | `ResourceLimit<R, Q>` |
| Non-releasable cumulative consumption | `ResourceBudget<R, Q>` |
| Releasable capacity | `ResourcePool<R, Q>` |
| Structured failure facts | `BudgetError<R, Q>`: `LimitExceeded`, `Insufficient`, `InvalidRelease` |
| Generic nested-data limits | `StructureLimits<R, Q>` and `StructureBudget<R, Q>` |
| JSON input/output and traversal limits (`json`) | `JsonLimits<R, Q>`, `JsonBudget<R, Q>`, and `JsonResource` |
| Explicit or clock-backed time limits (`time`) | `DurationBudget<R>` and `TimeBudget<R, C>` |

Quantities are exact unsigned values. Generic resource budgets default to `u64`,
while structural and JSON helpers default to `usize` because their measurements
are normally lengths, counts, and depths.

## Boundaries

- The core accounting APIs do not parse JSON, perform I/O, allocate output, or
  decide when a consumer should make a check. The opt-in `serde-json` feature
  supplies budget-aware Serde JSON adapters; it still does not choose a
  consumer's limits or domain error policy.
- It does not choose default limits, retry policy, redaction policy, scheduling,
  or application-specific error types.
- `BudgetError` reports mechanism facts, not a universal domain error. A
  consumer should map its resource and variant at its own public boundary.
- `ResourcePool` is a finite, non-synchronizing capacity object. It does not
  provide waiting, fairness, permits, cancellation, or concurrent access.
- `DurationBudget` consumes only durations explicitly submitted by its caller.
  `TimeBudget` observes an injected monotonic clock, so operation, waiting,
  queueing, and backoff all consume the same deadline.

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
