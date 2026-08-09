# `qubit-budget` User Guide

`qubit-budget` separates finite accounting mechanics from domain policy. Every
budget object represents a configured finite limit; an unconfigured dimension is
represented by `Option::None` and does not create an unlimited object. Quantities
are always `u64`.

Use `ResourceLimit` for a point observation, `ResourceBudget<R>` for a
non-releasable cumulative quantity, and `ResourcePool<R>` for capacity that can
be acquired, released and reused. `ResourceBudget` stores `remaining` and
subtracts only after a complete request fits, so failed requests are atomic and
cannot overflow an accumulated counter. `ResourcePoolError<R>` is shared by
acquisition and release, allowing one caller error type to use `?` for both:

```rust
fn acquire_then_release<R: Clone>(
    pool: &mut qubit_budget::ResourcePool<R>,
    amount: u64,
) -> Result<(), qubit_budget::ResourcePoolError<R>> {
    pool.try_acquire(amount)?;
    pool.release(amount)?;
    Ok(())
}
```

With the optional `time` feature, `DurationBudget<R>` accounts only for
explicitly submitted active durations. It never reads a clock. `TimeBudget<R,
C>` fixes a deadline in an injected `qubit-clock` monotonic domain, so operation,
waiting, queueing and backoff all consume the same continuous end-to-end budget.

The library does not choose whether bytes, nodes, depth, or properties are
bounded, and it does not choose defaults or domain error policy. Wire, parser,
redaction, I/O, retry and conversion crates should keep those policies locally
and translate structured budget facts into their existing public errors.
