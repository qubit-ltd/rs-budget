# qubit-budget user guide

[中文用户手册](user_guide.zh_CN.md) · [README](../README.md) ·
[API documentation](https://docs.rs/qubit-budget)

This guide applies to `qubit-budget` 0.3.x and Rust 1.94 or newer. Its examples
distill real uses in Qubit crates while leaving out unrelated application code.

## What problem does this crate solve?

At a boundary, “too large” can mean different things. A HTTP response can be
too many bytes; a configuration tree can be too deep; a directory walk can
hold too many handles; a retry flow can have spent too many attempts or too
much time. A counter alone does not answer the important follow-up questions:
did this failed operation change state, and which limit rejected it?

`qubit-budget` provides small accounting primitives. The caller still decides
what to measure, where the boundary is, and how to turn a typed failure into an
application error.

## Real Qubit crate use cases

| Crate | Protected work | Primitive used | Why it fits |
| --- | --- | --- | --- |
| `qubit-http` | Bytes collected from a streaming HTTP response | `ResourceBudget` | Each accepted chunk permanently consumes response-body capacity. |
| `qubit-config` | Source bytes, properties, nodes, and included sources | `ResourceBudget`, `ResourceLimit`, grouped charge | A child source must satisfy both its own and every ancestor’s limit. |
| `qubit-local-files` | Open directory readers during a copy or walk | `ResourcePool` | A reader occupies capacity while open, then explicitly returns it. |
| `qubit-retry` | Retry count, cumulative operation time, and total elapsed time | `ResourceBudget`, `DurationBudget`, `TimeBudget` | These are three different lifetimes, so they are accounted separately. |
| `qubit-json` | Raw input, normalized input, and one decoded JSON value | JSON sessions and transactions | I/O already accepted must remain charged; a failed value must not consume structural capacity. |

## The five accounting models

| Need | Type | Success | Failure |
| --- | --- | --- | --- |
| Check one fact without spending capacity | `ResourceLimit` | No mutation | No mutation; error carries the observed value and maximum. |
| Spend capacity that cannot return | `ResourceBudget` | `used` rises and `remaining` falls | The budget is unchanged. |
| Charge several limits as one decision | `ResourceBudget::try_consume_group` | Every budget is charged | No budget is charged. |
| Reuse capacity that has a real release event | `ResourcePool` | Acquire or release changes occupancy | The pool is unchanged. |
| Bound time | `DurationBudget` or `TimeBudget` | Records consumed duration or checks a clock deadline | Does not change a rejected check. |

Use a resource identity `R` that your application can show in errors and
metrics. Quantities `Q` are exact unsigned integers (`u8` through `u128`, or
`usize`). An optional limit set to `None` is unconfigured; it is not a hidden
unlimited budget.

## Scenario 1: `qubit-http` reads a bounded response body

`qubit-http` first rejects an oversized `Content-Length` hint when one is present.
It still needs a budget while reading chunks, because a server can omit or lie
about that header. The following is the central pattern from
`HttpResponse::read_body`, with networking and error mapping removed:

```rust
use qubit_budget::ResourceBudget;

let body_limit = 1_048_576_usize;
let mut body = Vec::new();
let mut body_budget = ResourceBudget::new("response body", body_limit);

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

## Scenario 2: `qubit-config` enforces local and parent limits together

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
depth.check(current_depth)?;
# Ok::<(), qubit_budget::LimitExceededError<&str, usize>>(())
```

## Scenario 3: `qubit-local-files` reuses directory-handle capacity

Copying a directory tree can require several directory readers at once.
`qubit-local-files` uses a `ResourcePool` for open readers: acquire once a reader
opens, then release exactly once it closes. Its `CopyBudget` follows this
pattern:

```rust
use qubit_budget::ResourcePool;

let mut directories = ResourcePool::new("open directory", 32_usize);
directories.try_acquire(1)?;

// Read this directory and possibly descend into it.

directories.release(1)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

Unlike a `ResourceBudget`, capacity returns after `release`. Releasing more
than was acquired is rejected and does not change the pool. The downstream
directory walker can also apply a `Reopen` policy: when the pool is full, it
closes retained readers, releases their slots, and retries acquisition.

Use a pool only when the application has a trustworthy, explicit release event.
It does not provide locking or RAII permits; wrap it in your own synchronization
or ownership model when those are required.

## Scenario 4: `qubit-retry` keeps three independent limits

`qubit-retry` composes three primitives in `RetryBudget`:

1. `ResourceBudget<RetryResource, u32>` admits a finite number of attempts.
2. `DurationBudget` tracks cumulative time spent inside completed operations.
3. `TimeBudget` uses `qubit-clock` to enforce one end-to-end monotonic deadline,
   including backoff and observer work.

The public `qubit-retry` API exposes those three limits as one `RetryBudget`:

```rust
use std::time::Duration;

use qubit_clock::StdMonotonicClock;
use qubit_retry::RetryBudget;
use qubit_retry::RetryPolicy;

let clock = StdMonotonicClock::new();
let policy = RetryPolicy::builder()
    .max_attempts(3)
    .max_operation_elapsed(Duration::from_secs(10))
    .max_total_elapsed(Duration::from_secs(30))
    .build()?;
let mut budget = RetryBudget::new(&clock, *policy.limits())?;

let attempt = budget.begin_attempt()?;
// Run one request attempt here.
let snapshot = budget.finish_attempt(attempt);

budget.check_retry_after(Duration::from_millis(500))?;
assert_eq!(snapshot.attempts(), 1);
# Ok::<(), Box<dyn std::error::Error>>(())
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
qubit-budget = { version = "0.3", features = ["time"] }
```

## Scenario 5: `qubit-json` makes one decoded value transactional

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

let mut session = JsonDecodeSession::owned(
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
use qubit_budget::json::JsonMeasurement;

let mut attempt = session.begin_value();
attempt.try_consume_input_bytes(raw.len())?;
attempt.try_consume_normalized_input_bytes(normalized.len())?;
attempt.try_admit(JsonMeasurement::String {
    depth: 1,
    bytes: 5,
})?;

// Parse, normalize, and deserialize the complete value here.
attempt.commit();
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

## Specialized helpers

- `StructureLimits` and `StructureBudget` combine point limits for depth,
  sequence items, map entries, and key bytes with a cumulative node budget.
- `StringLimits` checks the UTF-8 byte length of one string.
- `BigIntegerLimits` and `BigDecimalLimits` are feature-gated limits for their
  numeric representations.
- `ResourceBudget::try_write_string` buffers rendered text and charges its
  bytes only after rendering and UTF-8 validation both succeed.

## Errors, limits, and a practical rule of thumb

Prefer typed error accessors to parsing `Display` text. `LimitExceededError`
reports an observed value and inclusive maximum; `InsufficientBudgetError`
reports `limit`, `remaining`, and `requested`; `ResourceReleaseError` reports
an invalid pool release. Native `usize` and `u64` measurement APIs return
`MeasuredBudgetError` when conversion to `Q` is not exact—do not cast and
truncate first.

Choose limits in the application layer; this crate has no universal safe
defaults. Use a limit for a fact, a budget for irreversible work, a grouped
budget when several scopes must agree, a pool when capacity genuinely returns,
and a transaction when only a complete unit of work may consume its value
allowance.

## Further reading

- [README](../README.md)
- [中文用户手册](user_guide.zh_CN.md)
- [API documentation](https://docs.rs/qubit-budget)
