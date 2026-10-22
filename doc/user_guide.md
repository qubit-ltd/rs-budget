# qubit-budget user guide

[中文用户手册](user_guide.zh_CN.md) · [README](../README.md) ·
[API documentation](https://docs.rs/qubit-budget)

This guide applies to `qubit-budget` 0.9.x and Rust 1.94 or newer. Its examples
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
| `qubit-retry` | Admitted attempts, cumulative operation time, and total elapsed time | `ResourceBudget` | Current retry accounting uses explicit `qubit-clock` samples and internal `Duration` state. |
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
qubit-budget = "0.9"
```

Enable only the integrations used by the application. For example, JSON value
sessions require the `json` feature:

```toml
[dependencies]
qubit-budget = { version = "0.9", features = ["json"] }
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
fn main() -> Result<(), Box<dyn std::error::Error>> {
use qubit_budget::ResourceBudget;

let mut body_budget = ResourceBudget::new("response body", 8_usize);
let mut body = Vec::new();
for chunk in [b"hello".as_slice(), b" world".as_slice()] {
    if body_budget.try_consume(chunk.len()).is_err() { break; }
    body.extend_from_slice(chunk);
}
assert_eq!(body, b"hello");
assert_eq!(body_budget.used(), 5);
Ok(())
}
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
fn main() -> Result<(), Box<dyn std::error::Error>> {
use qubit_budget::ResourceBudget;

let mut root_properties = ResourceBudget::new("root properties", 100_usize);
let mut child_properties = ResourceBudget::new("child properties", 10_usize);

ResourceBudget::try_consume_group(
    &mut [&mut root_properties, &mut child_properties],
    1,
)?;
Ok(())
}
```

If the child has no capacity, the root is not charged accidentally. The error
identifies the first rejecting budget with `index()` and exposes its original
`InsufficientBudgetError` with `source_error()`.

The same downstream session uses `ResourceLimit` for nesting depth. Depth is a
property of the current node, not a resource consumed by every later node:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
use qubit_budget::ResourceLimit;

let depth = ResourceLimit::new("nesting depth", 16_usize);
let current_depth = 3_usize;
depth.check(current_depth)?;
Ok(())
}
```

### Scenario 3: reuse directory-handle capacity

Copying a directory tree can require several directory readers at once.
`qubit-local-files` uses a `ManagedResourcePool` for open readers. Each reader
owns a permit, so normal completion, errors, and panic unwinding return capacity:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
use qubit_budget::ManagedResourcePool;

let directories = ManagedResourcePool::new("open directory", 32_usize);
let permit = directories.try_acquire(1)?;

// Read this directory and possibly descend into it.

drop(permit);
Ok(())
}
```

Unlike a `ResourceBudget`, capacity returns when a permit is dropped. The
downstream directory walker can also apply a `Reopen` policy: when the pool is
full, it closes retained readers, drops their permits, and retries acquisition.

Use `ResourcePool` when the application needs explicit checked `release` calls.
Use `ManagedResourcePool` when ownership should make release automatic. Both
represent finite capacity and neither waits for capacity or enforces fairness.

### Scenario 4: keep three independent limits

Current `qubit-retry` uses `ResourceBudget` for admitted attempts. It tracks
completed-operation and total elapsed time using explicit `qubit-clock`
samples and internal `Duration` state. The following `DurationBudget` example
illustrates composition available in `qubit-budget`; it is not the current
internal implementation of `RetryBudget`.

The self-contained example shows how an application can compose attempt count
and operation duration:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
use std::time::Duration;

use qubit_budget::DurationBudget;
use qubit_budget::ResourceBudget;

let mut attempts = ResourceBudget::new("retry attempts", 3_u32);
let mut active_work = DurationBudget::new("active work", Duration::from_secs(10));
attempts.try_consume(1)?;
active_work.try_consume(Duration::from_secs(2))?;
assert_eq!(attempts.used(), 1);
assert_eq!(active_work.remaining(), Duration::from_secs(8));
Ok(())
}
```

`begin_attempt()` consumes an admitted attempt after the applicable checks.
`finish_attempt(now)` records the completed operation's elapsed duration.
`retry_limit(delay)` compares accumulated operation elapsed time and
`snapshot().total_elapsed().saturating_add(delay)` with their limits.

An attempt that ran too long is still recorded as completed; the accumulated
elapsed-time comparison can then reject a retry or its proposed delay. A
`TimeBudget` separately provides strict look-ahead deadline checks when an
application needs that behavior; it is not the implementation described for
current `RetryBudget`.

Enable the `time` feature to use `TimeBudget`:

```toml
[dependencies]
qubit-budget = { version = "0.9", features = ["time"] }
```

### Scenario 5: make one decoded JSON value transactional

`NormalizingJsonDecoder::new(policy, session)` owns the supplied accounting
session. `decode_str` prepares and decodes input. Preparation charges raw and
normalized input immediately; decoding a precharged document stages value
usage without charging those input bytes again. This retains I/O work already
done while allowing a failed value's staged node and payload usage to roll back.

For encoding, [`JsonEncoder::write_buffered`](https://docs.rs/qubit-json/latest/qubit_json/encode/struct.JsonEncoder.html#method.write_buffered)
buffers a complete document before writing. [`JsonEncoder::write_incremental`](https://docs.rs/qubit-json/latest/qubit_json/encode/struct.JsonEncoder.html#method.write_incremental)
streams output; if the writer accepts a prefix and a later write fails, that
prefix remains in the destination and its output bytes remain charged even when
the attempt is dropped. Choose the buffered entry point when the destination
must receive output only after the complete document is ready.

For JCS, validate the original input against I-JSON before converting it to a
`serde_json::Value`. A value-based encoder cannot detect duplicate object names
that parsing already discarded, or invalid Unicode escapes that parsing rejected.
JCS output budgeting limits the canonical bytes; it does not replace those
input checks.

The decoder starts an attempt, charges raw input before normalization, charges
normalized input during normalization, admits JSON measurements while scanning,
and calls `commit()` only after deserialization succeeds. The simplified
session setup is:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
use qubit_budget::json::JsonDecodeLimits;
use qubit_budget::json::JsonDecodeSession;
use qubit_budget::json::JsonResource;

let session = JsonDecodeSession::from_limits(
    JsonDecodeLimits::<JsonResource, usize>::builder()
        .max_input_bytes(64 * 1024)
        .max_normalized_input_bytes(64 * 1024)
        .max_depth(32)
        .max_nodes(10_000)
        .build(),
);
assert_eq!(session.value_budget().used_nodes(), Some(0));
Ok(())
}
```

For an adapter that reports measurements itself, the value part looks like this:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
use qubit_budget::json::JsonDecodeLimits;
use qubit_budget::json::JsonDecodeSession;
use qubit_budget::json::JsonMeasurement;
use qubit_budget::json::JsonResource;

let mut session = JsonDecodeSession::from_limits(
    JsonDecodeLimits::<JsonResource, usize>::builder()
        .max_input_bytes(100).max_normalized_input_bytes(100)
        .max_nodes(4).build(),
);
let raw = br#"{"name":"Ada"}"#;
assert_eq!(raw.len(), 14);
let mut attempt = session.begin_value();
attempt.try_consume_input_bytes(raw.len())?;
attempt.try_consume_normalized_input_bytes(raw.len())?;
attempt.try_admit(JsonMeasurement::Object { depth: 1, entries: 1 })?;
attempt.try_admit(JsonMeasurement::Key { bytes: 4 })?;
attempt.try_admit(JsonMeasurement::String { depth: 2, bytes: 3 })?;
attempt.commit()?;
assert_eq!(session.input_budget().unwrap().used(), 14);
assert_eq!(session.value_budget().used_nodes(), Some(2));
println!("accepted input=14 nodes=2");
Ok(())
}
```


Dropping an attempt keeps accepted input bytes and discards staged value usage:

```rust
use qubit_budget::json::{JsonDecodeLimits, JsonDecodeSession, JsonMeasurement, JsonResource};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut session = JsonDecodeSession::from_limits(
        JsonDecodeLimits::<JsonResource, usize>::builder().max_input_bytes(8).max_nodes(2).build(),
    );
    {
        let mut attempt = session.begin_value();
        attempt.try_consume_input_bytes(2)?;
        attempt.try_admit(JsonMeasurement::Null { depth: 1 })?;
    }
    assert_eq!(session.input_budget().unwrap().used(), 2);
    assert_eq!(session.value_budget().used_nodes(), Some(0));
    Ok(())
}
```

The first rejected admission is retained. Later admissions and `commit` return that error, and no staged node is published:

```rust
use qubit_budget::json::{JsonDecodeLimits, JsonDecodeSession, JsonMeasurement, JsonResource};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut session = JsonDecodeSession::from_limits(
        JsonDecodeLimits::<JsonResource, usize>::builder().max_nodes(0).build(),
    );
    let mut attempt = session.begin_value();
    assert!(attempt.try_admit(JsonMeasurement::Null { depth: 1 }).is_err());
    assert!(attempt.try_admit(JsonMeasurement::Null { depth: 1 }).is_err());
    assert!(attempt.commit().is_err());
    assert_eq!(session.value_budget().used_nodes(), Some(0));
    Ok(())
}
```

An owned session can use a resource identity borrowed from a local `String`; keep that string alive while the session is used:

```rust
use qubit_budget::json::{JsonDecodeLimits, JsonDecodeSession};
use qubit_budget::ResourceLimit;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let label = String::from("local input");
    let limits = JsonDecodeLimits::<&str, usize>::builder()
        .input_bytes_limit(ResourceLimit::new(label.as_str(), 2)).build();
    let mut session = JsonDecodeSession::from_limits(limits);
    let mut attempt = session.begin_value();
    attempt.try_consume_input_bytes(2).expect("borrowed label remains valid");
    attempt.commit().expect("attempt commits");
    assert_eq!(session.input_budget().unwrap().used(), 2);
    Ok(())
}
```

For object traversal, admit the object once, check its prospective entry count, then admit its key and value without admitting another object:

```rust
use qubit_budget::json::{JsonContainerKind, JsonMeasurement, JsonResource, JsonValueLimits};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut budget = JsonValueLimits::<JsonResource, usize>::builder()
        .max_nodes(2).max_map_entries(1).build().budget();
    let mut transaction = budget.transaction();
    transaction.try_enter_container(JsonContainerKind::Map, 1)?;
    transaction.check_container_count(JsonContainerKind::Map, 1)?;
    transaction.try_admit(JsonMeasurement::Key { bytes: 4 })?;
    transaction.try_admit(JsonMeasurement::String { depth: 2, bytes: 3 })?;
    transaction.commit()?;
    assert_eq!(budget.used_nodes(), Some(2));
    Ok(())
}
```

For an adapter that already accounts for normalized input, borrow that input budget and a separate value budget:

```rust
use qubit_budget::ResourceBudget;
use qubit_budget::json::JsonDecodeSession;
use qubit_budget::json::JsonMeasurement;
use qubit_budget::json::JsonResource;
use qubit_budget::json::JsonValueLimits;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut normalized = ResourceBudget::new(JsonResource::NormalizedInputBytes, 8_usize);
    let mut value = JsonValueLimits::builder().max_nodes(2).build().budget();
    {
        let mut session =
            JsonDecodeSession::borrowing_normalized_input(&mut normalized, &mut value);
        assert!(session.input_budget().is_none());
        let mut attempt = session.begin_value();
        attempt.try_consume_normalized_input_bytes(4)?;
        attempt.try_admit(JsonMeasurement::Null { depth: 1 })?;
        attempt.commit()?;
    }
    assert_eq!(normalized.used(), 4);
    assert_eq!(value.used_nodes(), Some(1));
    Ok(())
}
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

The owned `from_limits` session can retain a resource identity that borrows
local application data; keep that data alive until the session is no longer
used. The compiler enforces this lifetime.

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
truncate first. `QuantityMeasurement::AtLeastU64` reports a proven lower bound
when the original value cannot fit the narrow quantity; it does not claim an
exact measurement.

## Troubleshooting

- If a rejected operation unexpectedly changed state, check whether the work
  was an immediate charge (such as accepted input or writer output) rather
  than a staged JSON value admission.
- If a group charge reports a failure, use `BudgetGroupError::index()` and
  `source_error()` to inspect the first budget that rejected the amount; do
  not retry by charging its members one at a time.
- If a native `usize` or `u64` measurement fails conversion, keep the original
  value for diagnostics and select a quantity type `Q` that represents the
  application’s range exactly. When an error carries `AtLeastU64`, treat it as
  a lower bound and inspect `target()` to see which quantity type rejected it.

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
