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

JSON-specific limits, sessions, traversal, decoding, encoding, and Serde
errors are provided by `qubit-json`. A JSON boundary therefore imports generic
limits from `qubit_budget` and all JSON APIs from `qubit_json`.
