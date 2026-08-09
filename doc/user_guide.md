# `qubit-budget` User Guide

`qubit-budget` separates reusable accounting mechanics from domain policy.
`ResourceLimit` stores one immutable maximum, `ResourceBudget` stores remaining
capacity for one operation, and `LimitExceeded<K>` preserves a resource kind
chosen by the calling crate.

The core crate permits a zero maximum and represents an unbounded limit with
`usize::MAX`. `try_consume` leaves the budget unchanged when an amount does not
fit; use `check_additional` when a non-mutating check is needed.

`ResourceBudget::try_consume` keeps the budget unchanged when an amount does not
fit. `consume_or_exhaust` clears the remaining capacity before returning the
error, `consume_available` consumes as much as possible, and `release` returns
previously consumed capacity. `ResourceBudget` is not cloneable or copyable so
one accounting state cannot accidentally be duplicated.

The library does not choose whether bytes, nodes, depth, or properties are
bounded, and it does not choose defaults or error text. Wire, parser, redaction,
I/O, and conversion crates should keep those policies locally and translate the
typed facts into their existing public errors.
