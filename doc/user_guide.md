# `qubit-budget` User Guide

[中文用户指南](user_guide.zh_CN.md) | [README](../README.md) | [API documentation](https://docs.rs/qubit-budget)

`qubit-budget` separates finite accounting mechanics from domain policy. It is
for library authors who need a parser, decoder, or traversal to stop at a
bounded resource limit while preserving the caller's resource names and public
error model. Every budget object represents a configured finite limit; use
`Option::None` for an unconfigured dimension. Quantities are `u64` by default,
while structural and JSON helpers use `usize`.

## Concepts

`ResourceLimit<R, Q>` is an immutable inclusive maximum for one observation:
`check(actual)` either succeeds or returns `BudgetError::LimitExceeded`.
`ResourceBudget<R, Q>` records non-releasable cumulative consumption;
`ResourcePool<R, Q>` supports acquisition and release of reusable capacity.
Their failures are all `BudgetError<R, Q>`: `Insufficient` reports a request
that does not fit the remaining capacity, and `InvalidRelease` reports a
release that exceeds the amount in use.

`StructureLimits` packages optional limits for generic nested data and creates
independent `StructureBudget` sessions. `JsonLimits` and `JsonBudget`, behind
the `json` feature, apply the same model to JSON-specific resource identities.
Neither the generic structural types nor the JSON feature parse data.

## JSON traversal scenario

Assume a service already has a JSON parser and wants its traversal to reject a
root depth above 64 or more than 100,000 visited nodes. Enable `json`, create
one session for each input, call point checks when the parser observes a value,
and charge a node whenever the traversal processes one:

```rust
use qubit_budget::JsonLimits;

let limits = JsonLimits::new()
    .with_max_depth(64)
    .with_max_nodes(100_000);
let mut budget = limits.budget();
budget.check_depth(1)?;
budget.charge_node()?;
```

The complete input byte length, root-inclusive depth, array items, object
entries, decoded UTF-8 string byte length, and numeric lexical byte length are
point limits. Repeating an accepted point check does not consume any balance.
Nodes are cumulative: each `charge_node()` consumes one unit from that session,
and a failed charge leaves it unchanged. Calling `limits.budget()` for the next
input restores the configured node capacity for that new session.

`JsonLimits` is deliberately not a parser and the `json` feature includes no
Serde integration. The consuming parser remains responsible for identifying
the measurements and scheduling the checks.

## Error mapping

The crate exposes structured facts rather than a domain-wide error policy. At
the parser or wire boundary, map `BudgetError` to the error already exposed by
that crate. For example, a caller can use `JsonResource` and the error variant
to distinguish a depth violation from exhausted cumulative nodes.

## Other budgets

Use `StructureLimits` when the traversal is not specifically JSON. Its depth,
sequence-item, and map-entry limits are point checks; its node limit is
cumulative exactly as in `JsonBudget`. For a one-dimensional domain quantity,
use `ResourceLimit`, `ResourceBudget`, or `ResourcePool` directly. The optional
`time` feature provides `DurationBudget<R>` for explicitly submitted active
durations and `TimeBudget<R, C>` for one monotonic deadline that includes
operation, waiting, queueing, and backoff time.

## Limits and best practices

This crate does not choose byte, node, depth, or property limits; it provides
no JSON parser, Serde integration, I/O, redaction, default limits, retry
policy, or application error type. Configure limits at the owning boundary,
create a fresh structure or JSON budget for each independently bounded input,
and translate `BudgetError` there.
