# qubit-budget user guide

[中文用户指南](user_guide.zh_CN.md) · [README](../README.md) ·
[API documentation](https://docs.rs/qubit-budget)

`qubit-budget` models finite, monotonic resource consumption. Use
`ResourceLimit` for immutable configuration and `ResourceBudget` where a
single resource charge must retain remaining capacity. A failed
`try_consume` call does not change that budget.

## JSON accounting model

Enable the `json` feature for JSON resource accounting. `JsonValueLimits`
configures traversal and payload limits; its `JsonValueBudget` stores only
committed usage. `JsonValueTransaction` holds the working state for one value.
Use `JsonMeasurement` to describe native JSON events rather than mutating
individual node or payload counters.

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

`try_admit` is atomic for one measurement. It leaves the transaction usable
after rejection. `commit` publishes the whole working state; ordinary drop,
error propagation, and panic unwinding discard that working state without
changing the committed value budget.

## Decode and encode attempts

Callers create each attempt explicitly with `begin_value()` for one complete
top-level value. The attempt owns immediate I/O charges and one value
transaction. This makes the rollback boundary explicit instead of treating the
entire session as a snapshot.

```rust
use qubit_budget::json::JsonDecodeLimits;
use qubit_budget::json::JsonDecodeSession;
use qubit_budget::json::JsonMeasurement;

let mut session = JsonDecodeSession::owned(
    JsonDecodeLimits::empty()
        .with_max_input_bytes(32)
        .with_max_normalized_input_bytes(32)
        .with_max_nodes(8),
);
let mut attempt = session.begin_value();
attempt.try_consume_input_bytes(5)?;
attempt.try_consume_normalized_input_bytes(5)?;
attempt.try_admit(JsonMeasurement::String { depth: 1, bytes: 5 })?;
attempt.commit();
# Ok::<(), qubit_budget::MeasuredBudgetError<qubit_budget::json::JsonResource, usize>>(())
```

For encoding, use `JsonEncodeSession::owned` or `JsonEncodeSession::borrowing_output`,
then call `begin_value()`. `JsonEncodeAttempt::try_consume_output_bytes` charges
only bytes the calling encoder knows were accepted. Both attempt types publish
value accounting only through `commit`.

## Atomicity matrix

| Scenario | Input | Normalized input | Value | Output |
| --- | --- | --- | --- | --- |
| Strict decode succeeds | retained | not applicable | committed | not applicable |
| Strict decode fails | retained | not applicable | rolled back | not applicable |
| Lenient decode fails | retained | retained | rolled back | not applicable |
| Buffered `Vec<u8>` output fails | not applicable | not applicable | rolled back | success-only; no `Vec` means no output charge |
| Buffered writer partially fails | not applicable | not applicable | rolled back | each accepted prefix is retained immediately |
| Incremental writer fails | not applicable | not applicable | rolled back | each accepted prefix is retained immediately |
| One value in a stream fails | retained across values | retained across values | only the current value rolls back | previously accepted output remains retained |

Raw input and normalized input are immediate, retained charges. A complete
`Vec<u8>` may be charged only after successful serialization, so a failed
operation that returns no `Vec` has no output charge. A buffered or incremental
writer is different: every accepted prefix is charged immediately and remains
charged after a later budget, serialization, I/O, or panic failure.

Each stream value has an independent attempt: a failed later value cannot roll
back previously committed values. Higher-level grouping may deliberately use
one transaction for a larger business operation, and business code may handle a
rejection, continue admitting other measurements, and commit when that larger
operation succeeds.

## External effects and limits

A value transaction rolls back only value accounting. Dropping an attempt or
transaction cannot undo bytes already accepted by a writer, callback effects,
`Hasher` updates, or object mutation. If a business operation needs those
effects to remain unchanged after a budget rejection, it must preflight before
performing them or use a domain-specific recovery strategy.

This crate supplies limits and accounting; parsing, normalization, traversal,
Serde adapters, and application error policies remain in
[`qubit-json`](https://crates.io/crates/qubit-json). At an untrusted boundary,
configure the raw-input, normalized-input, value, and output dimensions that
apply to the operation.
