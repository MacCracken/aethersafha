# Architecture Decision Records

Decisions about aethersafha — what we chose, the context, and the consequences we accept. Use these when a future reader would reasonably ask *"why did we do it this way?"*

## Conventions

- **Filename**: `NNNN-kebab-case-title.md`, zero-padded to four digits. Never renumber.
- **One decision per ADR.** If a decision supersedes a prior one, add a new ADR and set the old one's status to `Superseded by NNNN`.
- **Status lifecycle**: `Proposed` → `Accepted` → (optionally) `Superseded` or `Deprecated`.
- Use [`template.md`](template.md) as the starting point.

## ADR vs. architecture note vs. guide

| Kind | Lives in | Answers |
|---|---|---|
| ADR | `docs/adr/` | *Why did we choose X over Y?* |
| Architecture note | `docs/architecture/` | *What non-obvious constraint is true about the code?* |
| Guide | `docs/guides/` | *How do I do X?* |

## Index

| ADR | Title | Status |
|---|---|---|
| [0001](0001-native-display-protocol.md) | Native display protocol (Wayland refused) | Accepted |
| [0002](0002-retire-the-rust-protocol-layer.md) | Retire the Rust protocol layer, and record what it carried | Accepted |
| [0003](0003-the-ports-divergences-from-rust.md) | The port's divergences from Rust: substrate, representation, and accepted behaviour | Accepted |
| [0004](0004-the-retired-surface.md) | The retired surface: what the port does not carry over from Rust | Accepted |
