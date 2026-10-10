# Architecture notes

Non-obvious constraints, quirks, and invariants that a reader cannot derive from the code alone. Numbered chronologically — never renumber.

Not decisions (those live in [`../adr/`](../adr/)) and not guides (those live in [`../guides/`](../guides/)). An item here describes *how the world is*, not *what we chose* or *how to do something*.

## Items

- [001 — What a setu client can count on from aethersafha](001-setu-client-contract.md) — how a client is started, the environment it gets, which keys reach it, and the largest surface it can attach.
- [002 — Where the Rust original's public API lives now](002-rust-public-api-map.md) — every `pub mod` and `pub use` in the Rust `lib.rs`, the example and the Cargo files, each mapped to its Cyrius home or the ADR that retired it. The Rust tree itself is at tag `0.17.5`.

Add the next as `003-kebab-case-title.md`. Do not write entries for decisions — those are ADRs.
