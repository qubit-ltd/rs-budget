# qubit-budget design

[中文设计文档](design.zh_CN.md) · [User guide](user_guide.md) · [README](../README.md)

This document records the stable accounting semantics behind `qubit-budget` 0.4.x.

## Layered model

`ResourceLimit` checks one observed value without mutation. `ResourceBudget` spends finite capacity permanently; `ResourcePool` represents capacity that callers release; and `ManagedResourcePool` binds release to a permit's Drop lifetime. Higher-level structure, string, duration, and value helpers compose those primitives without changing their rejected-charge rule.

JSON has two independent ledgers. Decode and encode sessions retain accepted I/O immediately. A `JsonValueTransaction` stages only one value's structural and payload measurements, and publishes that staged state only through `commit`.

## Atomicity and state transitions

| Operation | Immediate state | Staged value state | On Drop or failed outer operation |
| --- | --- | --- | --- |
| Accepted raw/normalized input | retained | unchanged | retained |
| Accepted writer output prefix | retained | unchanged | retained |
| Accepted value measurement | unchanged | staged | discarded unless committed |
| Successful `commit` | unchanged | published | committed usage remains |
| Rejected value measurement | unchanged | poisoned | all staged usage discarded |

Dropping a transaction cannot undo a callback, object mutation, hasher update, or an accepted output prefix. Adapters that need a wider business transaction choose that boundary themselves.

## Poison and error priority

The first failed value admission permanently poisons its transaction. Later admissions and `commit` return the retained error without publishing staged usage. I/O failures do not poison a value transaction by themselves, because they describe work already observed rather than an invalid staged value.

When a grouped budget charge fails, `BudgetGroupError` identifies the first rejecting member and preserves its source error. Callers should inspect typed accessors such as `index`, `source_error`, `remaining`, and `requested` rather than parsing display text.

## Features and downstream boundary

The default crate stays dependency-light. `json`, `time`, `big-integer`, and `big-decimal` are opt-in; `big-decimal` also enables `big-integer`. Feature-gated modules and re-exports display their required feature on docs.rs.

Downstream crates choose resource identities, numerical limits, parsing, I/O, waiting, synchronization, and recovery policy. They must not assume whole-operation rollback from a value transaction, clone finite budgets, or rely on error display text as a stable interface.
