# `qubit-budget` User Guide

[中文用户指南](user_guide.zh_CN.md) | [README](../README.md) | [API documentation](https://docs.rs/qubit-budget)

This guide covers `qubit-budget` 0.4 for library authors who need one parser,
decoder, encoder, or traversal to stop at explicit resource boundaries while
preserving their own resource names and public errors.

## Conceptual model

`ResourceLimit<R, Q>` is an immutable inclusive maximum for one observation;
`ResourceBudget<R, Q>` records non-releasable cumulative consumption; and
`ResourcePool<R, Q>` models reusable capacity. A point-limit failure is
`BudgetError::LimitExceeded`, while a cumulative request that does not fit is
`BudgetError::Insufficient`. Releasing more than a pool currently has in use
returns the separate `ResourceReleaseError`; it is not a `BudgetError`.

JSON accounting has three layers:

- `JsonValueLimits` and `JsonValueBudget` cover direction-independent
  structure, nodes, keys, strings, numbers, and cumulative payload.
- `JsonDecodeLimits` and `JsonDecodeSession` add cumulative input bytes.
- `JsonEncodeLimits` and `JsonEncodeSession` add cumulative output bytes.

Limits are immutable configuration. A session is mutable state for one
operation. `Option::None` represents an unconfigured dimension.

The default `StructureLimits` and `StructureBudget` use `usize`, matching Rust
collection lengths and counts. JSON value, string, and big-number helpers use
`u64` where stable cross-target wire measurements are more important than the
native collection type.

## Scenario: admit a request and bound its response

Assume an endpoint accepts a small JSON request and returns compact JSON. The
success criterion is that the request is admitted before typed decoding and
the complete response never exceeds its output policy.

## Installation and minimal configuration

```toml
[dependencies]
qubit-budget = { version = "0.4", features = ["serde-json"] }
serde = { version = "1.0", features = ["derive"] }
```

Build the shared value policy once, then embed it in independent directional
limits:

```rust
use qubit_budget::JsonDecodeLimits;
use qubit_budget::JsonEncodeLimits;
use qubit_budget::JsonResource;
use qubit_budget::JsonValueLimits;
use qubit_budget::ResourceLimit;
use qubit_budget::StructureLimits;

let structure = StructureLimits::empty()
    .with_depth_limit(ResourceLimit::new(JsonResource::Depth, 8))
    .with_nodes_limit(ResourceLimit::new(JsonResource::Nodes, 128));
let values = JsonValueLimits::empty()
    .with_structure_limits(structure)
    .with_payload_bytes_limit(ResourceLimit::new(
        JsonResource::PayloadBytes,
        4096,
    ));
let decode_limits = JsonDecodeLimits::empty()
    .with_input_bytes_limit(ResourceLimit::new(JsonResource::InputBytes, 4096))
    .with_value_limits(values);
let encode_limits = JsonEncodeLimits::empty()
    .with_output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, 4096))
    .with_value_limits(values);
```

## Core workflow

The caller owns the sessions and passes them into the Serde adapters. This is
the external admission boundary: `decode_slice` charges the complete input
before lexical validation and typed deserialization. `encode_to_vec` checks
the value and charges bytes as compact output is produced.

```rust
use qubit_budget::decode_slice;
use qubit_budget::encode_to_vec;
use qubit_budget::JsonDecodeLimits;
use qubit_budget::JsonDecodeSession;
use qubit_budget::JsonEncodeLimits;
use qubit_budget::JsonEncodeSession;
use qubit_budget::JsonResource;
use qubit_budget::ResourceLimit;
use serde::Deserialize;
use serde::Serialize;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
#[derive(Debug, Deserialize, Serialize)]
struct Request {
    name: String,
}

let input = br#"{"name":"Ada"}"#;
let decode_limits = JsonDecodeLimits::empty().with_input_bytes_limit(
    ResourceLimit::new(JsonResource::InputBytes, 64),
);
let mut decode_session = JsonDecodeSession::owned(decode_limits);
let request: Request = decode_slice(input, &mut decode_session)?;

let encode_limits = JsonEncodeLimits::empty().with_output_bytes_limit(
    ResourceLimit::new(JsonResource::OutputBytes, 64),
);
let mut encode_session = JsonEncodeSession::owned(encode_limits);
let output = encode_to_vec(&request, &mut encode_session)?;
assert_eq!(output, input);
# Ok(())
# }
```

Use `encode_to_writer` when the final destination implements `Write`. Budget
and Serde failures leave that destination untouched because the adapter first
buffers the accepted document. An I/O failure during the final `write_all` can
still leave partial output because `Write` has no rollback operation.

## Advanced usage

With only the `json` feature, drive `JsonValueBudget` and the directional
sessions from another parser. Use `StructureLimits` for non-JSON nested data,
or `ResourceLimit`, `ResourceBudget`, and `ResourcePool` for one-dimensional
resources. `DurationBudget` is always available; the `time` feature adds the
monotonic-clock `TimeBudget` and its error type.

## Errors and diagnostics

The crate reports accounting facts, not an application-wide error policy. Map
`BudgetError` or `ResourceReleaseError` at the domain boundary. A rejected point
check or cumulative request does not change that dimension, but consumption
accepted earlier in the operation remains committed. `decode_slice` therefore
keeps input bytes consumed even if later lexical or typed decoding fails.

## Troubleshooting

- If a second document unexpectedly fails, check whether the same session was
  reused. Construct a fresh session for an independent operation.
- If depth passes but nodes fail, remember that depth is a point limit while
  nodes are cumulative.
- If the external writer contains a prefix after failure, inspect the I/O error;
  budget and Serde failures occur before the final write.

## Limitations and best practices

This crate does not choose default byte, node, depth, retry, or redaction
policies and does not define an application's public error type. Configure
limits at the owning boundary. Reuse immutable limits freely, create fresh
sessions for independently bounded operations, and reuse a session only when
cumulative accounting across calls is intentional.

The `serde-json` adapter retains a small compatibility layer for the private
serializer shapes used by `serde_json`'s arbitrary-precision number and raw-value
support. Its nonrecursive lexical preflight runs before typed decoding. The
repository includes `cargo fuzz` targets for differential decode checks and
budget invariants; fuzz execution requires a nightly toolchain.

## Further reading

- [README](../README.md)
- [中文用户指南](user_guide.zh_CN.md)
- [API documentation](https://docs.rs/qubit-budget)
