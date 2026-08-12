# qubit-budget

`qubit-budget` provides dependency-light, format-agnostic finite resource
limits and accounting primitives for Qubit Rust crates.

It owns generic `ResourceLimit`, `ResourceBudget`, `ResourcePool`,
`StructureLimits`, `StructureBudget`, and string, numeric, duration, and time
helpers. A budget is always finite; use `Option<ResourceBudget<_, _>>` for an
unconfigured dimension.

## Features

| Feature | Adds |
| --- | --- |
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

JSON traversal, parsing, Serde adapters, sessions, and JSON resource limits
belong to [`qubit-json`](https://crates.io/crates/qubit-json). Keep generic
resource and structure limits in `qubit-budget`, and import all `Json*` types
and JSON encoding/decoding APIs from `qubit_json`.

## Boundaries

This crate does not parse JSON, perform I/O, allocate output, select limits,
or define application-specific error policies.
