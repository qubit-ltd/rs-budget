# qubit-budget user guide

[中文用户手册](user_guide.zh_CN.md) · [README](../README.md) ·
[API documentation](https://docs.rs/qubit-budget)

This guide applies to `qubit-budget` 0.4.x and Rust 1.94 or newer.

## Purpose and audience

Use this crate when a parser, serializer, converter, or I/O boundary must place
finite limits on work and report exactly why an operation was rejected. The
guide is for library authors and service developers who need reusable
accounting rules without coupling them to a particular parser or runtime.

The crate supplies limits, mutable accounting, and structured errors. Your
integration remains responsible for measuring events, performing I/O, choosing
policy values, and deciding whether to retry, reject, or recover.

## Conceptual model

Every constraint combines a caller-defined resource identity `R` with a
quantity `Q`:

- `R` appears in errors, so an application can distinguish resources such as
  request bytes, JSON nodes, or open files.
- `Q` is an exact unsigned integer (`u8`, `u16`, `u32`, `u64`, `u128`, or
  `usize`). Native `usize` and `u64` measurements can be converted without
  truncation through the measured APIs.
- An unconfigured dimension is `None`. A budget or pool always means that a
  finite constraint is configured.

Choose the state model that matches the resource:

| Need | Type | Successful operation | Failed operation |
| --- | --- | --- | --- |
| Check one independent value | `ResourceLimit` | No mutation | Returns the observed value and maximum |
| Spend a non-reusable allowance | `ResourceBudget` | Reduces `remaining` | Leaves the budget unchanged |
| Reuse explicitly returned capacity | `ResourcePool` | Acquire or release changes `available` | Leaves the pool unchanged |

`ResourceBudget` is intentionally not `Clone`: cloning it would duplicate a
finite allowance. `ResourcePool` does not synchronize access, wait, issue RAII
permits, or provide fairness.

Composite helpers add two more boundaries:

- `StructureLimits` combines point limits such as depth or container size with
  a cumulative node budget.
- JSON sessions keep raw/normalized input and accepted output as immediate
  charges, while a `JsonValueTransaction` stages structural and payload usage
  until `commit`.

## Scenario: protect an untrusted JSON request

Assume a gateway accepts one JSON document. Its success criteria are:

- at most 64 raw input bytes and 64 normalized input bytes;
- root-inclusive depth at most 3;
- at most 8 nodes in total;
- at most 16 UTF-8 bytes in one string and 32 payload bytes in total;
- value usage is published only after the parser has completed the document.

`qubit-budget` does not parse the bytes. The parser or adapter measures accepted
input and emits `JsonMeasurement` events while it traverses the value.

## Installation and feature selection

The core limit, budget, pool, structure, string, and duration types require no
feature:

```toml
[dependencies]
qubit-budget = "0.4"
```

Enable JSON accounting for the scenario:

```toml
[dependencies]
qubit-budget = { version = "0.4", features = ["json"] }
```

Optional features are:

| Feature | Public capability |
| --- | --- |
| `json` | JSON resources, limits, value transactions, and decode/encode sessions |
| `big-integer` | `BigIntegerLimits` for `num_bigint::BigInt` |
| `big-decimal` | `BigDecimalLimits`; also enables `big-integer` |
| `time` | Clock-backed `TimeBudget` and `TimeBudgetError` |

## Core workflow

### 1. Configure one owned decode session

`JsonDecodeLimits::empty()` starts with every dimension unconfigured. Add only
the dimensions enforced by this boundary:

```rust
use qubit_budget::json::JsonDecodeLimits;
use qubit_budget::json::JsonDecodeSession;
use qubit_budget::json::JsonMeasurement;

let mut session = JsonDecodeSession::owned(
    JsonDecodeLimits::empty()
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
assert_eq!(attempt.used_payload_bytes(), Some(5));
attempt.commit();

assert_eq!(session.input_budget().expect("configured input").used(), 7);
assert_eq!(session.value_budget().used_nodes(), Some(1));
# Ok::<(), qubit_budget::MeasuredBudgetError<qubit_budget::json::JsonResource, usize>>(())
```

Depth is root-inclusive. `String::bytes` and object-key byte counts are UTF-8
byte lengths; number bytes are the length of the representation seen by the
adapter. `PayloadBytes` accumulates keys, strings, and numbers.

### 2. Admit events in parser order

Call `try_admit` for every JSON event:

| Parsed event | Measurement |
| --- | --- |
| `null` | `JsonMeasurement::Null { depth }` |
| Boolean | `JsonMeasurement::Boolean { depth }` |
| String | `JsonMeasurement::String { depth, bytes }` |
| Number | `JsonMeasurement::Number { depth, bytes }` |
| Array | `JsonMeasurement::Array { depth, items }` |
| Object | `JsonMeasurement::Object { depth, entries }` |
| Object key | `JsonMeasurement::Key { bytes }` |

Each admission first validates conversions and point limits, then checks
cumulative node and payload capacity. If it fails, that one admission does not
change the transaction. The transaction remains usable if the surrounding
business operation intentionally handles the rejection.

Use `check_container_count(JsonContainerKind::Sequence, prospective)` or the
`Map` variant when a streaming parser must reject the next child before
entering it. This check is non-mutating.

### 3. Commit only a complete value

Call `commit` after the parser, normalization, validation, and any other work
inside the chosen value boundary has succeeded. Dropping the attempt—normally,
through `?`, or during panic unwinding—discards staged value usage.

Input accounting is deliberately different. Once raw or normalized bytes have
been accepted by an attempt, those charges remain even if its value transaction
is dropped. This records work the decoder already performed.

For a stream, normally create one attempt per complete top-level value. A later
bad value then rolls back only its own value usage; earlier commits and all
accepted input remain visible in the session.

Callers create each attempt explicitly with `begin_value()`.

### Atomicity matrix

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
a wider business boundary, while `commit` remains the only way to publish the
staged value state.

## Using the core primitives directly

### Point limits

Use `ResourceLimit` for an independent observation such as one message depth:

```rust
use qubit_budget::ResourceLimit;

let depth = ResourceLimit::new("message-depth", 3_usize);
depth.check(3).expect("the inclusive maximum fits");

let error = depth.check(4).expect_err("depth four is rejected");
assert_eq!(error.resource(), &"message-depth");
assert_eq!(error.exact_observed(), Some(4));
assert_eq!(error.maximum(), 3);
```

### Cumulative budgets and grouped charges

Use `ResourceBudget` when consumption cannot be returned. A grouped charge
checks every member before changing any of them:

```rust
use qubit_budget::ResourceBudget;

let mut request = ResourceBudget::new("request-bytes", 5_u64);
let mut tenant = ResourceBudget::new("tenant-bytes", 2_u64);

let error = ResourceBudget::try_consume_group(
    &mut [&mut request, &mut tenant],
    3,
)
.expect_err("the tenant budget rejects the charge");

assert_eq!(error.index(), 1);
assert_eq!(request.remaining(), 5);
assert_eq!(tenant.remaining(), 2);
```

`consume_available` is intentionally partial: it consumes
`min(requested, remaining)` and returns the amount actually consumed. Always
use its return value.

### Releasable capacity

Use `ResourcePool` only when callers explicitly return capacity:

```rust
use qubit_budget::ResourcePool;

let mut files = ResourcePool::new("open-files", 2_u64);
files.try_acquire(2).expect("both slots are available");
files.release(1).expect("one acquired slot can be returned");

assert_eq!(files.available(), 1);
assert_eq!(files.in_use(), 1);
```

Releasing more than `in_use` returns `ResourceReleaseError` and does not change
the pool.

## Advanced usage

### Borrow budgets across integrations

Owned JSON sessions are convenient for one isolated operation. Use
`borrowing_input`, `borrowing_all`, `borrowing_output`, or `borrowing_value`
when several adapters must charge caller-owned budgets. The caller then defines
the real accounting lifetime—for example, one budget shared by all values in a
request stream.

### Choose the correct encode strategy

`JsonEncodeSession` separates output accounting from value accounting:

- If the serializer returns a complete `Vec<u8>`, serialize first, call
  `check_output_bytes`, and charge the complete length only after serialization
  succeeds. No returned `Vec` means no output was accepted.
- If a writer accepts output incrementally, call
  `try_consume_output_bytes` for each accepted prefix. Those charges remain
  after a later serialization, I/O, budget, or panic failure.
- In both cases, call `try_admit` for value events and `commit` only when the
  complete value succeeds.

An attempt is a value-accounting boundary, not a general side-effect rollback.
Dropping it cannot undo writer output, callbacks, `Hasher` updates, or object
mutation.

### Render a string transactionally

`ResourceBudget::try_write_string` buffers UTF-8 output and charges the budget
only after rendering and UTF-8 validation succeed:

```rust
use std::fmt::Write as _;

use qubit_budget::ResourceBudget;

let mut output = ResourceBudget::new("output-bytes", 8_u64);
let rendered = output
    .try_write_string(|writer| {
        write!(writer.as_fmt(), "id={}", 42)
    })
    .expect("five bytes fit");

assert_eq!(rendered, "id=42");
assert_eq!(output.used(), 5);
```

The closure can instead use `writer.as_io()`. Budget rejection, renderer error,
allocation failure, invalid UTF-8, or length/conversion failure leaves the
budget unchanged.

### Other reusable helpers

- `StructureLimits` and `StructureBudget` cover depth, cumulative nodes,
  per-sequence items, per-map entries, and key bytes without depending on JSON.
- `StringLimits` checks the UTF-8 byte length of one string.
- `BigIntegerLimits` and `BigDecimalLimits` check numeric representation
  properties when their features are enabled.
- `DurationBudget` tracks a caller-supplied duration allowance.
- `TimeBudget` uses `qubit-clock` to check elapsed time and deadlines when the
  `time` feature is enabled.

## Errors and diagnostics

| Error | Meaning | State on failure |
| --- | --- | --- |
| `LimitExceededError` | One observation exceeded an inclusive maximum | No mutation |
| `InsufficientBudgetError` | A cumulative charge or pool acquisition exceeded remaining capacity | No mutation |
| `BudgetGroupError` | One member rejected an all-or-nothing group charge | No member is charged; `index()` identifies the first rejection |
| `QuantityConversionError` | A native measurement did not fit `Q` exactly | No mutation |
| `MeasuredBudgetError` | Wraps conversion or budget failure for native measurements | No mutation for the rejected admission |
| `ResourceReleaseError` | A pool release exceeded current `in_use` | No mutation |
| `BudgetedStringError` | Rendering, allocation, UTF-8, length, conversion, or budget failure | String budget is unchanged |

Use the typed accessors instead of parsing `Display` text. Resource identities
are available on both point and cumulative failures. `Observation::Exact`
contains an exact value; `Observation::AtLeast` is a safe lower bound used when
an integration can prove only that the maximum was crossed.

## Troubleshooting

### A limit appears to do nothing

Confirm that the dimension was configured. `empty()` and `unconfigured()`
create sets with every optional limit absent. Also confirm that the integration
emits the corresponding measurement; the crate cannot observe parser or writer
activity by itself.

### JSON value usage stays at zero

`try_admit` changes only the transaction's working state. Call `commit` on the
attempt after the complete value succeeds. Inspect `attempt.used_nodes()` before
commit and `session.value_budget().used_nodes()` afterward.

### Input or output usage remains after an error

This is expected for accepted I/O. Decode attempts retain raw and normalized
input charges. Writer-oriented encode attempts retain accepted output prefixes.
Only staged value accounting rolls back on drop.

### A measurement returns `MeasuredBudgetError::Quantity`

The native `usize` or `u64` value does not fit the selected `Q`. Use a wider
unsigned quantity type or lower the measured input; never cast and truncate the
value before checking it.

### A pool release fails

Compare `requested()` with `in_use()`. `ResourcePool` does not track ownership
or return capacity automatically, so the integration must pair successful
acquisitions with valid explicit releases.

## Limitations and best practices

- Choose limits in the application layer; this crate supplies no safe universal
  defaults.
- Configure every dimension that matters at an untrusted boundary. A raw-input
  limit alone does not constrain normalized expansion, nesting, node count,
  payload, or output.
- Keep resource identities stable and meaningful so logs and metrics can group
  failures without parsing messages.
- Use `check_*` for preflight and `try_*` for mutation. Do not treat a preflight
  as a reservation if other code can mutate the same accounting object.
- Do not share mutable budgets across threads without external synchronization.
  The crate provides accounting semantics, not a concurrency protocol.
- JSON transactions use fixed-size accounting state, but the crate does not
  bound allocations or side effects performed by the surrounding parser,
  serializer, or application.
- Run one attempt per intended rollback boundary and document which external
  effects are immediate.

## Further reading

- [Project README](../README.md)
- [中文用户手册](user_guide.zh_CN.md)
- [API documentation](https://docs.rs/qubit-budget)
- [`qubit-json` on crates.io](https://crates.io/crates/qubit-json)
- [Source repository](https://github.com/qubit-ltd/rs-budget)
