# qubit-budget user guide

[中文用户手册](user_guide.zh_CN.md) · [README](../README.md) ·
[API documentation](https://docs.rs/qubit-budget)

This guide applies to `qubit-budget` 0.4.x and Rust 1.94 or newer. Its examples
distill real uses in Qubit crates while leaving out unrelated application code.

## Purpose and audience

At a boundary, “too large” can mean different things. A HTTP response can be
too many bytes; a configuration tree can be too deep; a directory walk can
hold too many handles; a retry flow can have spent too many attempts or too
much time. A counter alone does not answer the important follow-up questions:
did this failed operation change state, and which limit rejected it?

`qubit-budget` provides small accounting primitives for library and service
authors. The caller still decides what to measure, where the boundary is, and
how to turn a typed failure into an application error.

## Conceptual model

### Representative Qubit integration patterns

| Crate | Protected work | Primitive used | Why it fits |
| --- | --- | --- | --- |
| `qubit-http` | Bytes collected from a streaming HTTP response | `ResourceBudget` | Each accepted chunk permanently consumes response-body capacity. |
| `qubit-config` | Source bytes, properties, nodes, and included sources | `ResourceBudget`, `ResourceLimit`, grouped charge | A child source must satisfy both its own and every ancestor’s limit. |
| `qubit-local-files` | Open directory readers during a copy or walk | `ManagedResourcePool` | A reader owns a permit while open; Drop returns capacity. |
| `qubit-retry` | Retry count, cumulative operation time, and total elapsed time | `ResourceBudget`, `DurationBudget`, `TimeBudget` | These are three different lifetimes, so they are accounted separately. |
| `qubit-json` | Raw input, normalized input, and one decoded JSON value | JSON sessions and transactions | I/O already accepted must remain charged; a failed value must not consume structural capacity. |

### Accounting models

| Need | Type | Success | Failure |
| --- | --- | --- | --- |
| Check one fact without spending capacity | `ResourceLimit` | No mutation | No mutation; error carries the observed value and maximum. |
| Spend capacity that cannot return | `ResourceBudget` | `used` rises and `remaining` falls | The budget is unchanged. |
| Account generic structure | `StructureBudget` | Each accepted measurement is charged immediately | Only the rejected measurement is unchanged. |
| Charge several limits as one decision | `ResourceBudget::try_consume_group` | Every budget is charged | No budget is charged. |
| Reuse capacity with explicit release | `ResourcePool` | Acquire or release changes occupancy | The pool is unchanged. |
| Reuse capacity through an owned lifetime | `ManagedResourcePool` | Returns a Drop-based permit | The pool is unchanged. |
| Bound time | `DurationBudget` or `TimeBudget` | Records consumed duration or checks a clock deadline | Does not change a rejected check. |

Use a resource identity `R` that your application can show in errors and
metrics. Quantities `Q` are exact unsigned integers (`u8` through `u128`, or
`usize`). An optional limit set to `None` is unconfigured; it is not a hidden
unlimited budget.

## Installation and minimal configuration

Add the crate with no feature enabled for the core limits, budgets, and pools:

```toml
[dependencies]
qubit-budget = "0.4"
```

Enable only the integrations used by the application. For example, JSON value
sessions require the `json` feature:

```toml
[dependencies]
qubit-budget = { version = "0.4", features = ["json"] }
```

The crate supports Rust 1.94 or newer. Its optional features are `json`,
`big-integer`, `big-decimal` (which also enables `big-integer`), and `time`.

## Scenario and core workflow: read a bounded response body

`qubit-http` first rejects an oversized `Content-Length` hint when one is present.
It still needs a budget while reading chunks, because a server can omit or lie
about that header. The following is the central pattern from
`HttpResponse::read_body`, with networking and error mapping removed:

`response_chunks` stands for the chunks supplied by the application's HTTP
client. Success means every accepted chunk fits and `body` contains their
concatenation; rejection leaves the current chunk out of both `body` and the
budget.

```rust
use qubit_budget::ResourceBudget;

let body_limit = 1_048_576_usize;
let mut body = Vec::new();
let mut body_budget = ResourceBudget::new("response body", body_limit);
let response_chunks = [b"hello".as_slice(), b" world".as_slice()];

for chunk in response_chunks {
    body_budget.try_consume(chunk.len())?;
    body.extend_from_slice(&chunk);
}
# Ok::<(), qubit_budget::InsufficientBudgetError<&str, usize>>(())
```

The order matters: `try_consume` runs before the chunk is appended. If a
chunk would cross the limit, neither the budget nor the accumulated body is
changed by that chunk. `qubit-http` uses `used() + requested()` to
report the size that was observed at the rejection.

This is a **cumulative budget**. Do not use `ResourceLimit` here: each chunk
may be individually small while their sum is too large.

## Advanced usage

### Scenario 2: enforce local and parent limits together

Configuration sources may include other sources. `qubit-config` gives every source
its own byte, property, node, and source-count budgets, while also borrowing
the corresponding budgets of all ancestors. When a child emits a property,
every active scope must accept it or none may be charged:

```rust
use qubit_budget::ResourceBudget;

let mut root_properties = ResourceBudget::new("root properties", 100_usize);
let mut child_properties = ResourceBudget::new("child properties", 10_usize);

ResourceBudget::try_consume_group(
    &mut [&mut root_properties, &mut child_properties],
    1,
)?;
# Ok::<(), qubit_budget::BudgetGroupError<&str, usize>>(())
```

If the child has no capacity, the root is not charged accidentally. The error
identifies the first rejecting budget with `index()` and exposes its original
`InsufficientBudgetError` with `source_error()`.

The same downstream session uses `ResourceLimit` for nesting depth. Depth is a
property of the current node, not a resource consumed by every later node:

```rust
use qubit_budget::ResourceLimit;

let depth = ResourceLimit::new("nesting depth", 16_usize);
let current_depth = 3_usize;
depth.check(current_depth)?;
# Ok::<(), qubit_budget::LimitExceededError<&str, usize>>(())
```

### Scenario 3: reuse directory-handle capacity

Copying a directory tree can require several directory readers at once.
`qubit-local-files` uses a `ManagedResourcePool` for open readers. Each reader
owns a permit, so normal completion, errors, and panic unwinding return capacity:

```rust
use qubit_budget::ManagedResourcePool;

let directories = ManagedResourcePool::new("open directory", 32_usize);
let permit = directories.try_acquire(1)?;

// Read this directory and possibly descend into it.

drop(permit);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Unlike a `ResourceBudget`, capacity returns when a permit is dropped. The
downstream directory walker can also apply a `Reopen` policy: when the pool is
full, it closes retained readers, drops their permits, and retries acquisition.

Use `ResourcePool` when the application needs explicit checked `release` calls.
Use `ManagedResourcePool` when ownership should make release automatic. Both
represent finite capacity and neither waits for capacity or enforces fairness.

### Scenario 4: keep three independent limits

`qubit-retry` composes three primitives in `RetryBudget`:

1. `ResourceBudget<RetryResource, u32>` admits a finite number of attempts.
2. `DurationBudget` tracks cumulative time spent inside completed operations.
3. `TimeBudget` uses `qubit-clock` to enforce one end-to-end monotonic deadline,
   including backoff and observer work.

The following self-contained example shows the two explicit accounting
dimensions that underlie that composition:

```rust
use std::time::Duration;

use qubit_budget::DurationBudget;
use qubit_budget::ResourceBudget;

let mut attempts = ResourceBudget::new("retry attempts", 3_u32);
let mut active_work = DurationBudget::new("active work", Duration::from_secs(10));
attempts.try_consume(1)?;
active_work.try_consume(Duration::from_secs(2))?;
assert_eq!(attempts.used(), 1);
assert_eq!(active_work.remaining(), Duration::from_secs(8));
# Ok::<(), qubit_budget::InsufficientBudgetError<&str, u32>>(())
```

`begin_attempt()` first checks all continuation limits, then consumes one
attempt. `finish_attempt()` records the elapsed operation time even when the
operation exceeded its allowance. `check_retry_after()` tests the proposed
backoff against the total deadline before sleeping.

This separation is deliberate. A request can finish an attempt that took too
long; that completed attempt is an observed fact. The overrun consumes the
operation-duration allowance, so `begin_attempt()` rejects a later retry.
Separately, `check_retry_after(delay)` rejects a delay that would reach the
total deadline. This is why a duration allowance is not interchangeable with a
clock deadline.

Enable the `time` feature to use `TimeBudget`:

```toml
[dependencies]
qubit-budget = { version = "0.4", features = ["time"] }
```

### Scenario 5: make one decoded JSON value transactional

`qubit-json` normalizes and decodes untrusted JSON through
`NormalizingJsonDecoder::decode_with_session`. It must retain I/O work that already
happened, but must not spend a value’s node and payload allowance if parsing or
deserialization later fails.

The decoder starts an attempt, charges raw input before normalization, charges
normalized input during normalization, admits JSON measurements while scanning,
and calls `commit()` only after deserialization succeeds. The simplified
session setup is:

```rust
use qubit_budget::json::JsonDecodeLimits;
use qubit_budget::json::JsonDecodeSession;
use qubit_budget::json::JsonResource;

let mut session = JsonDecodeSession::from_limits(
    JsonDecodeLimits::<JsonResource, usize>::builder()
        .max_input_bytes(64 * 1024)
        .max_normalized_input_bytes(64 * 1024)
        .max_depth(32)
        .max_nodes(10_000)
        .build(),
);
```

For an adapter that reports measurements itself, the value part looks like this:

```rust
use qubit_budget::json::JsonDecodeLimits;
use qubit_budget::json::JsonDecodeSession;
use qubit_budget::json::JsonMeasurement;
use qubit_budget::json::JsonResource;

let mut session = JsonDecodeSession::from_limits(
    JsonDecodeLimits::<JsonResource, usize>::builder()
        .max_input_bytes(64 * 1024)
        .max_normalized_input_bytes(64 * 1024)
        .max_nodes(10_000)
        .build(),
);
let raw = br#"{\"name\":\"Ada\"}"#;
let normalized = raw;
let mut attempt = session.begin_value();
attempt.try_consume_input_bytes(raw.len())?;
attempt.try_consume_normalized_input_bytes(normalized.len())?;
attempt.try_admit(JsonMeasurement::String {
    depth: 1,
    bytes: 5,
})?;

// Parse, normalize, and deserialize the complete value here.
attempt.commit()?;
# Ok::<(), qubit_budget::MeasuredBudgetError<qubit_budget::json::JsonResource, usize>>(())
```

### Complete atomicity matrix

| Scenario | Input | Normalized input | Value | Output |
| --- | --- | --- | --- | --- |
| Strict decode succeeds | retained | not applicable | committed | not applicable |
| Strict decode fails | retained | not applicable | rolled back | not applicable |
| Lenient decode fails | retained | retained | rolled back | not applicable |
| Buffered `Vec<u8>` output fails | not applicable | not applicable | rolled back | success-only; no `Vec` means no output charge |
| Buffered writer partially fails | not applicable | not applicable | rolled back | each accepted prefix is retained immediately |
| Incremental writer fails | not applicable | not applicable | rolled back | each accepted prefix is retained immediately |
| One value in a stream fails | retained across values | retained across values | only the current value rolls back | previously accepted output remains retained |

Raw input and normalized input are immediate charges. Dropping a transaction
cannot undo an accepted prefix, callback effect, `Hasher` update, or object
mutation. A higher-level operation may intentionally choose one transaction for
a wider business boundary, while only `commit` publishes staged value usage.
Callers create each attempt explicitly with `begin_value()`.

The first failed value admission poisons the transaction. Every later
admission and `commit` return that retained error, while a poisoned `commit`
publishes no staged value state. Dropping it rolls back all staged value usage.
Raw or normalized input failures and writer I/O failures do not independently
poison the value transaction; accepted I/O charges and output prefixes remain
immediate.

## Specialized helpers

- `StructureLimits` and `StructureBudget` combine point limits for depth,
  sequence items, map entries, and key bytes with a cumulative node budget.
- `StringLimits` checks the UTF-8 byte length of one string.
- `BigIntegerLimits` and `BigDecimalLimits` are feature-gated limits for their
  numeric representations.
- `ResourceBudget::try_write_string` buffers rendered text and charges its
  bytes only after rendering and UTF-8 validation both succeed.

## Errors and diagnostics

Prefer typed error accessors to parsing `Display` text. `LimitExceededError`
reports an observed value and inclusive maximum; `InsufficientBudgetError`
reports `limit`, `remaining`, and `requested`; `ResourceReleaseError` reports
an invalid pool release. Native `usize` and `u64` measurement APIs return
`MeasuredBudgetError` when conversion to `Q` is not exact—do not cast and
truncate first.

## Troubleshooting

- If a rejected operation unexpectedly changed state, check whether the work
  was an immediate charge (such as accepted input or writer output) rather
  than a staged JSON value admission.
- If a group charge reports a failure, use `BudgetGroupError::index()` and
  `source_error()` to inspect the first budget that rejected the amount; do
  not retry by charging its members one at a time.
- If a native `usize` or `u64` measurement fails conversion, keep the original
  value for diagnostics and select a quantity type `Q` that represents the
  application’s range exactly.

## Limitations and best practices

Choose limits in the application layer; this crate has no universal safe
defaults. Use a limit for a fact, a budget for irreversible work, a grouped
budget when several scopes must agree, a pool when capacity genuinely returns,
and a transaction when only a complete unit of work may consume its value
allowance.

The crate does not parse data, perform I/O, wait for pool capacity, synchronize
`ResourcePool`, choose application limits, or define recovery policy.
`ManagedResourcePool` synchronizes its own accounting and returns permits on
Drop, but it also does not wait for capacity or guarantee fairness.

## Further reading

- [README](../README.md)
- [中文用户手册](user_guide.zh_CN.md)
- [Design document](design.md)
- [API documentation](https://docs.rs/qubit-budget)
