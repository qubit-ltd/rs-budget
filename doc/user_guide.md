# qubit-budget user guide

[中文用户手册](user_guide.zh_CN.md) · [README](../README.md) ·
[API documentation](https://docs.rs/qubit-budget)

This guide applies to `qubit-budget` 0.4.x and Rust 1.94 or newer. It is for
library authors and service developers who must bound untrusted or expensive
work without tying their policy to one parser, serializer, or runtime.

## The model: limit, budget, or pool?

Every constraint has a resource label `R` and an exact unsigned quantity `Q`.
The label appears in errors; `Q` can be `u8`, `u16`, `u32`, `u64`, `u128`, or
`usize`. `None` means that one optional dimension is not configured—it does
not mean that the crate has created a hidden unlimited budget.

| If you need to… | Use | Key rule |
| --- | --- | --- |
| Check one observation | `ResourceLimit` | It never changes state. |
| Spend a finite allowance | `ResourceBudget` | A failed `try_*` charge leaves it unchanged. |
| Reuse returned capacity | `ResourcePool` | The integration must release capacity explicitly. |

`ResourceBudget` deliberately is not `Clone`, because cloning would duplicate
an allowance. `ResourcePool` is not a semaphore: it has no locking, waiting,
fairness, ownership tracking, or RAII permits.

## Scenario: accept one untrusted JSON document

Imagine a gateway that accepts one JSON document only when all of these rules
hold:

- no more than 64 raw-input bytes and 64 normalized-input bytes;
- root-inclusive depth no greater than 3;
- at most 8 nodes in total;
- a string no longer than 16 UTF-8 bytes, and no more than 32 payload bytes in
  total;
- JSON value usage becomes permanent only after the complete document succeeds.

`qubit-budget` does not parse JSON for you. Your parser or adapter reports the
input it accepted and emits one `JsonMeasurement` for each value or object key.

## Install and configure

The core types need no feature:

```toml
[dependencies]
qubit-budget = "0.4"
```

Enable `json` for the scenario:

```toml
[dependencies]
qubit-budget = { version = "0.4", features = ["json"] }
```

The other optional features are `big-integer`, `big-decimal` (which enables
`big-integer`), and `time`.

## Core workflow

### 1. Create one session for the boundary you own

Start with an owned session when its budgets belong to one decode operation:

```rust
use qubit_budget::json::JsonDecodeLimits;
use qubit_budget::json::JsonDecodeSession;
use qubit_budget::json::JsonMeasurement;
use qubit_budget::json::JsonResource;

let mut session = JsonDecodeSession::owned(
    JsonDecodeLimits::<JsonResource, usize>::new()
        .with_max_input_bytes(64)
        .with_max_normalized_input_bytes(64)
        .with_max_depth(3)
        .with_max_nodes(8)
        .with_max_string_bytes(16)
        .with_max_payload_bytes(32),
);

let mut attempt = session.begin_value();
attempt.try_consume_input_bytes(7)?;
attempt.try_consume_normalized_input_bytes(7)?;
attempt.try_admit(JsonMeasurement::String {
    depth: 1,
    bytes: 5,
})?;

assert_eq!(attempt.used_nodes(), Some(1));
attempt.commit();
assert_eq!(session.input_budget().expect("configured input").used(), 7);
assert_eq!(session.value_budget().used_nodes(), Some(1));
# Ok::<(), qubit_budget::MeasuredBudgetError<qubit_budget::json::JsonResource, usize>>(())
```

`new()` leaves every limit absent. Add only the dimensions enforced at this
boundary. Depth includes the root; string and key lengths are UTF-8 byte
lengths; payload counts key, string, and number bytes.

### 2. Tell the attempt what the parser observed

Call `try_admit` in parser order:

| Parser observation | Measurement |
| --- | --- |
| `null` | `JsonMeasurement::Null { depth }` |
| Boolean | `JsonMeasurement::Boolean { depth }` |
| String | `JsonMeasurement::String { depth, bytes }` |
| Number | `JsonMeasurement::Number { depth, bytes }` |
| Array | `JsonMeasurement::Array { depth, items }` |
| Object | `JsonMeasurement::Object { depth, entries }` |
| Object key | `JsonMeasurement::Key { bytes }` |

One admission validates conversion and point limits before cumulative node and
payload capacity. If it fails, that admission does not alter the attempt. A
streaming parser can non-mutatingly preflight a prospective child with
`check_container_count` on the value transaction.

### 3. Commit only after the whole value succeeds

Call `commit()` after parsing, normalization, validation, and all other work
inside your chosen value boundary have succeeded. Dropping the attempt—through
`?`, ordinary scope exit, or panic unwinding—rolls back only staged value use.

Input is different: raw and normalized bytes are charged as soon as the decoder
accepts them. For a stream, usually create one attempt per top-level value, so
a bad later value rolls back its own structural and payload usage but not earlier
commits or accepted input.

| Accounted work | When it becomes permanent |
| --- | --- |
| Raw and normalized decode input | Immediately when accepted |
| Output accepted by an incremental writer | Immediately per accepted prefix |
| Value nodes and payload | Only when `commit()` succeeds |

An attempt is an accounting boundary, not a general side-effect rollback. It
cannot undo writer output, callbacks, hasher updates, or mutations elsewhere.

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

## Use the core types directly

Use a point limit for one independent value:

```rust
use qubit_budget::ResourceLimit;

let depth = ResourceLimit::new("message-depth", 3_usize);
depth.check(3).expect("the inclusive maximum fits");

let error = depth.check(4).expect_err("depth four is rejected");
assert_eq!(error.exact_observed(), Some(4));
assert_eq!(error.maximum(), 3);
```

Use a budget when capacity cannot return. Grouped charges are all-or-nothing:

```rust
use qubit_budget::ResourceBudget;

let mut request = ResourceBudget::new("request-bytes", 5_u64);
let mut tenant = ResourceBudget::new("tenant-bytes", 2_u64);

let error = ResourceBudget::try_consume_group(&mut [&mut request, &mut tenant], 3)
    .expect_err("the tenant limit rejects the charge");

assert_eq!(error.index(), 1);
assert_eq!(request.remaining(), 5);
assert_eq!(tenant.remaining(), 2);
```

Use a pool only for capacity that your code will explicitly return:

```rust
use qubit_budget::ResourcePool;

let mut files = ResourcePool::new("open-files", 2_u64);
files.try_acquire(2).expect("both slots are available");
files.release(1).expect("one slot is returned");

assert_eq!(files.available(), 1);
assert_eq!(files.in_use(), 1);
```

`consume_available` is intentionally partial: it consumes the smaller of the
request and remaining capacity, then returns the amount consumed. Always use
that result.

## Further options

- Use `JsonDecodeSession::borrowing_value`, `borrowing_input`, or
  `borrowing_all` when several adapters must share caller-owned budgets.
- For encoding a complete `Vec<u8>`, serialize first and charge output only
  after success. For an incremental writer, charge every accepted prefix.
- `ResourceBudget::try_write_string` renders to a buffer and charges bytes only
  after successful rendering and UTF-8 validation.
- `StructureLimits` and `StructureBudget` provide non-JSON depth, node,
  container-size, and key-byte limits. `StringLimits`, numeric limits,
  `DurationBudget`, and `TimeBudget` cover the corresponding domains.

## Errors and diagnostics

Prefer typed accessors over parsing an error message:

| Error | Meaning | State after rejection |
| --- | --- | --- |
| `LimitExceededError` | One observation exceeded its inclusive maximum | Unchanged |
| `InsufficientBudgetError` | A budget charge or pool acquisition exceeded capacity | Unchanged |
| `BudgetGroupError` | One member rejected a grouped charge | No member is charged |
| `QuantityConversionError` | A native measurement cannot fit `Q` exactly | Unchanged |
| `MeasuredBudgetError` | Conversion or accounting failure from a measured API | Rejected admission is unchanged |
| `ResourceReleaseError` | A release exceeds `in_use` | Unchanged |

`Observation::Exact` carries an exact observed value. `Observation::AtLeast`
is a safe lower bound for integrations that can prove only that the maximum was
crossed.

## Troubleshooting and limits

| Symptom | Check first |
| --- | --- |
| A limit has no effect | Was the dimension configured, and does the adapter emit its measurement? |
| JSON value usage stays zero | Inspect the attempt, then call `commit()` after a successful value. |
| Input or output remains charged after an error | This is expected for accepted I/O; only staged value use rolls back. |
| `MeasuredBudgetError::Quantity` | Select a wider unsigned `Q`; do not truncate with a cast. |
| A pool release fails | Compare `requested()` with `in_use()` and pair successful acquisitions with releases. |

Choose actual limits in the application layer—there is no safe universal
default. Configure every relevant dimension at an untrusted boundary; a raw
input limit alone does not constrain normalized expansion, nesting, nodes,
payload, or output. Do not share mutable budgets between threads without
external synchronization. The crate has fixed-size accounting state, but it
cannot bound allocations or side effects performed by your parser, serializer,
or application.

## Further reading

- [README](../README.md)
- [中文用户手册](user_guide.zh_CN.md)
- [API documentation](https://docs.rs/qubit-budget)
- [`qubit-json` on crates.io](https://crates.io/crates/qubit-json)
