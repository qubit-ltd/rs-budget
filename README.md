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
| `json` | Directional `JsonDecodeLimits`/`JsonEncodeLimits`, shared `JsonValueLimits`, and operation sessions |
| `serde-json` | Budget-aware Serde JSON deserialization and serialization adapters; also enables `json` |
| `big-integer` | `BigIntegerLimits` for magnitude bits and significant decimal digits |
| `big-decimal` | `BigDecimalLimits` for coefficient and scale magnitude limits |
| `time` | Monotonic-clock `TimeBudget` and `TimeBudgetError` (`DurationBudget` is always available) |

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

Imagine a wire boundary that must admit an untrusted JSON document and then
emit a bounded response. Decode and encode use separate directional sessions:
input bytes can only be charged while decoding, output bytes can only be
charged while encoding, and both directions reuse the same value limits.

```rust
use qubit_budget::decode_slice;
use qubit_budget::encode_to_vec;
use qubit_budget::JsonDecodeLimits;
use qubit_budget::JsonDecodeSession;
use qubit_budget::JsonEncodeLimits;
use qubit_budget::JsonEncodeSession;
use qubit_budget::JsonResource;
use qubit_budget::JsonValueLimits;
use qubit_budget::ResourceLimit;
use qubit_budget::StructureLimits;
use serde::Deserialize;
use serde::Serialize;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
#[derive(Debug, Deserialize, Serialize)]
struct Document {
    items: Vec<String>,
}

let input = br#"{"items":["alpha"]}"#;

let structure = StructureLimits::empty()
    .with_depth_limit(ResourceLimit::new(JsonResource::Depth, 64))
    .with_nodes_limit(ResourceLimit::new(JsonResource::Nodes, 100_000))
    .with_sequence_items_limit(ResourceLimit::new(
        JsonResource::SequenceItems,
        10_000,
    ))
    .with_map_entries_limit(ResourceLimit::new(
        JsonResource::MapEntries,
        10_000,
    ))
    .with_key_bytes_limit(ResourceLimit::new(JsonResource::KeyBytes, 256));
let value_limits = JsonValueLimits::empty()
    .with_structure_limits(structure)
    .with_string_bytes_limit(ResourceLimit::new(
        JsonResource::StringBytes,
        256 * 1024,
    ))
    .with_number_bytes_limit(ResourceLimit::new(JsonResource::NumberBytes, 4_096))
    .with_payload_bytes_limit(ResourceLimit::new(
        JsonResource::PayloadBytes,
        512 * 1024,
    ));

let decode_limits = JsonDecodeLimits::empty()
    .with_input_bytes_limit(ResourceLimit::new(
        JsonResource::InputBytes,
        1_048_576,
    ))
    .with_value_limits(value_limits);
let mut decode_session = JsonDecodeSession::owned(decode_limits);
let document: Document = decode_slice(input, &mut decode_session)?;

let encode_limits = JsonEncodeLimits::empty()
    .with_output_bytes_limit(ResourceLimit::new(
        JsonResource::OutputBytes,
        1_048_576,
    ))
    .with_value_limits(value_limits);
let mut encode_session = JsonEncodeSession::owned(encode_limits);
let output = encode_to_vec(&document, &mut encode_session)?;
assert_eq!(output, input);
# Ok(())
# }
```

`decode_slice` first consumes the complete input length from the caller-owned
`JsonDecodeSession`, then performs lexical admission and typed Serde decoding.
`encode_to_vec` charges structure and emitted bytes online through the
`JsonEncodeSession`; `encode_to_writer` buffers the accepted document before
touching the destination. Create a new session for each independently bounded
operation. Reusing a session intentionally shares cumulative input/output,
node, and payload consumption across calls.

The same state rules make failure handling predictable. For example, a node
budget of one accepts its first charge and rejects the second without changing
the accepted count:

```rust
use qubit_budget::JsonResource;
use qubit_budget::JsonValueBudget;
use qubit_budget::JsonValueLimits;
use qubit_budget::ResourceLimit;
use qubit_budget::StructureLimits;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let structure = StructureLimits::empty()
    .with_nodes_limit(ResourceLimit::new(JsonResource::Nodes, 1));
let limits = JsonValueLimits::empty().with_structure_limits(structure);
let mut budget = JsonValueBudget::new(limits);
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
nodes, sequence items, map entries, and structural key bytes.
`JsonValueLimits<R, Q>` adds per-string, per-number, and cumulative payload
limits. `JsonDecodeLimits` then adds input bytes, while `JsonEncodeLimits` adds
output bytes. The corresponding session owns mutable operation state; immutable
limits can be reused to construct as many fresh sessions as needed.

An unconfigured dimension is represented by `Option::None`; it is not an
unlimited budget object that needs to be driven through every call. Point-limit
and cumulative-consumption failures do not roll back consumption accepted
earlier in the same operation. The rejected point check or cumulative request
itself is atomic and leaves that dimension unchanged.

## How Downstream Crates Use It

| Downstream crate | Actual use | What it demonstrates |
| --- | --- | --- |
| `rs-value` | Uses separate `JsonDecodeLimits` and `JsonEncodeLimits`, runs one directional session through each wire operation, and maps `BudgetError` to its wire errors. | Read and write policies cannot accidentally charge the wrong byte direction. |
| `rs-redact` | Shares operation-scoped input, output, and mask budgets across nested diagnostic renderers. | A child component cannot silently reset the allowance for the enclosing operation. |
| `rs-config` | Applies bounded JSON wire and interpolation budgets at configuration boundaries. | Configuration parsing and expansion share the same directional and cumulative accounting. |
| `rs-datatype` | Shares borrowed JSON value budgets with typed values and enforces string/number limits. | Nested typed values cannot reset the enclosing operation's value allowance. |
| `rs-http`, `rs-fs`, `rs-local-files` | Charges response bodies, streams, and file reads as data arrives, including unknown-length inputs. | The same cumulative invariant works independently of chunk boundaries and transport errors. |
| `rs-metadata` | Bounds metadata JSON wire operations with directional sessions. | Metadata boundaries use the same admission and output guarantees as application payloads. |
| `rs-retry` | Combines an attempt-count `ResourceBudget`, an explicit `DurationBudget`, and a continuous elapsed-time deadline. | Different resources can use different accounting semantics in one domain policy. |

## What It Provides

| Need | Public API |
| --- | --- |
| One inclusive point check | `ResourceLimit<R, Q>` |
| Non-releasable cumulative consumption | `ResourceBudget<R, Q>` |
| Releasable capacity | `ResourcePool<R, Q>` |
| Point/cumulative failure facts | `BudgetError<R, Q>`: `LimitExceeded`, `Insufficient` |
| Invalid pool release facts | `ResourceReleaseError<R, Q>` |
| Generic nested-data limits | `StructureLimits<R, Q>` and `StructureBudget<R, Q>` |
| JSON value traversal limits (`json`) | `JsonValueLimits<R, Q>` and `JsonValueBudget<R, Q>` |
| Directional JSON operations (`json`) | `JsonDecodeLimits`/`JsonDecodeSession` and `JsonEncodeLimits`/`JsonEncodeSession` |
| Explicit duration limits | `DurationBudget<R>` |
| Monotonic-clock time limits (`time`) | `TimeBudget<R, C>` and `TimeBudgetError` |

Quantities are exact unsigned values. Generic resource budgets remain generic,
while the default structural limits use `usize`, matching Rust collection sizes.
String, big-number, JSON value, and Serde JSON helpers use `u64` so limits have
the same meaning on 32-bit and 64-bit targets. JSON decode sessions
can be `owned(...)` or can borrow caller-owned input and value budgets with
`borrowing(...)`.

`BudgetError::LimitExceeded` reports `Observation::Exact` when the measurement
is known and `Observation::AtLeast` when only a safe lower bound is available;
consumers must not treat an `AtLeast` value as an exact measurement.

For non-JSON text, the same stable quantity semantics are available directly:

```rust
let limits = StringLimits::empty().with_utf8_bytes_limit(
    ResourceLimit::new(MyResource::TextBytes, 1024_u64),
);
limits.check(input)?;
```

The `serde-json` adapter keeps a small compatibility layer for the private
serializer shapes used by `serde_json`'s arbitrary-precision number and raw-value
support. It is intentionally nonrecursive and performs lexical admission before
typed decoding. The repository's `fuzz/` targets differentially compare decode
acceptance with `serde_json` and check session accounting invariants; run them
with `cargo fuzz` on a nightly toolchain.

## Boundaries

- The core accounting APIs do not parse JSON, perform I/O, allocate output, or
  decide when a consumer should make a check. The opt-in `serde-json` feature
  supplies budget-aware Serde JSON adapters; it still does not choose a
  consumer's limits or domain error policy.
- It does not choose default limits, retry policy, redaction policy, scheduling,
  or application-specific error types.
- `BudgetError` reports mechanism facts, not a universal domain error. A
  consumer should map its resource and variant at its own public boundary.
- Releasing more capacity than a `ResourcePool` currently has in use is not a
  budget exhaustion. `ResourcePool::release` returns the separate
  `ResourceReleaseError` type.
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
