# qubit-budget user guide

`qubit-budget` models finite, monotonic resource consumption. Use
`ResourceLimit` for an immutable limit and `ResourceBudget` when an operation
must retain remaining capacity. Failed `try_consume` calls are atomic.

```rust
use qubit_budget::ResourceBudget;

let mut bytes = ResourceBudget::new("bytes", 16_u64);
bytes.try_consume(4)?;
assert_eq!(bytes.used(), 4);
# Ok::<(), qubit_budget::BudgetError<&str>>(())
```

Use `StructureLimits` and `StructureBudget` for generic nested data. Use the
string and numeric helpers when their measurement semantics fit the boundary.
`ResourceBudget::clone` creates an independent valid snapshot, which is useful
for transactional adapters.

## JSON boundaries

Enable the `json` feature when the caller needs JSON resource accounting. The
`qubit-budget` crate owns `JsonResource`, `JsonValueLimits`,
`JsonDecodeLimits`, `JsonEncodeLimits`, and their sessions. Parsing,
normalization, traversal, and Serde adapters remain in `qubit-json`.

```rust
use qubit_budget::json::JsonDecodeLimits;

let limits = JsonDecodeLimits::empty()
    .with_max_input_bytes(1024)
    .with_max_normalized_input_bytes(2048)
    .with_max_depth(32)
    .with_max_nodes(256);
assert_eq!(limits.max_normalized_input_bytes(), Some(2048));
```

The lenient decoder maps its compatibility option
`JsonDecodeOptions::with_max_normalized_bytes` into the session's normalized
input budget before normalization. This keeps allocation admission and
accounting on one budget path.

## Testing and limits

Run `cargo test --all-features` for JSON and optional numeric/time helpers.
Unconfigured dimensions remain unlimited and are represented by `Option` in
the corresponding session; callers at untrusted boundaries should configure
raw input, normalized input, value, and output limits explicitly.
