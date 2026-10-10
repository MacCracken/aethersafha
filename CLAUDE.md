# aethersafha — Claude Code Instructions

> **Core rule**: this file is **preferences, process, and procedures** —
> durable rules that change rarely. Volatile state (current version,
> module line counts, port progress, test counts, consumers) lives in
> [`docs/development/state.md`](docs/development/state.md).
> Do not inline state here.

## Project Identity

**aethersafha** — Cyrius port of a Rust project (27207 lines of `src/**/*.rs`; the last tag that carries it, at `rust-old/`, is `0.17.5`).

- **Type**: Port (Rust → Cyrius)
- **License**: AGPL-3.0-only. Full user-facing projects, such as this desktop, are AGPL; component libraries
  are GPL-3.0-only (operator, 2026-10-09). Do not align this with the GPL siblings: `LICENSE`, `README.md`,
  `CONTRIBUTING.md` and `cyrius.cyml` all say AGPL-3.0-only.
- **Language**: Cyrius (toolchain pinned in `cyrius.cyml [package].cyrius`)
- **Version**: `VERSION` at the project root is the source of truth — do not inline the number here
- **Standards**: [First-Party Standards](https://github.com/MacCracken/agnosticos/blob/main/docs/development/applications/first-party-standards.md) · [First-Party Documentation](https://github.com/MacCracken/agnosticos/blob/main/docs/development/applications/first-party-documentation.md)

## Goal

_TODO: one-or-two-sentence mission statement. What does aethersafha OWN in the stack? Durable — doesn't change per release._

## Current State

> Volatile state lives in [`docs/development/state.md`](docs/development/state.md) —
> port progress, surface parity, in-flight work. Refreshed every release.

This file (`CLAUDE.md`) is durable rules.

## Scaffolding

Project was scaffolded with `cyrius port`. The original Rust was deleted at 0.17.6. The record is ADRs 0001–0004, the `.tcyr` suites, and tag `0.17.5` — cross-check the port against it.

**Citations.** A citation `rust-old/<path>[:<line>]`, or a bare `<name>.rs:<line>` in a parity comment, refers to that path at tag `0.17.5` — read it with `git show 0.17.5:rust-old/<path>`.

## Quick Start

```sh
cyrius deps                              # resolve dependencies
cyrius build src/main.cyr build/aethersafha    # compile
cyrius test                              # run tests/*.tcyr
```

## Key Principles

- **Cross-check against the record** (ADRs 0001–0004, the `.tcyr` suites, tag `0.17.5`) — the port's correctness bar is "matches what Rust did". Diverge only with an ADR.
- **Correctness over cleverness** — if the Cyrius behavior diverges silently from Rust, the bugs win
- Test after every change, not after the feature is "done"
- ONE change at a time — never bundle unrelated changes
- Build with `cyrius build`, not raw `cat file | cc5` — the manifest auto-resolves deps
- Source files only need project includes — stdlib auto-resolves from `cyrius.cyml`
- `var buf[N]` = N **bytes**, not N entries

## Rules (Hard Constraints)

- **Do not commit or push** — the user handles all git operations
- **Never use `gh` CLI** — use `curl` to the GitHub API if needed
- The parity record is ADRs 0001–0004, the `.tcyr` suites, and tag `0.17.5` (`git show 0.17.5:rust-old/...`) — the Rust original lives only in that tag
- Do not skip tests before claiming changes work
- Do not modify `lib/` files (vendored stdlib / dep symlinks)
- Do not hardcode toolchain versions in CI YAML — `cyrius = "X.Y.Z"` in `cyrius.cyml` is the source of truth
- **`cyrius.cyml` is a manifest — NOT a ledger, NOT a changelog.** A comment there says what a dep is for
  and any rule needed to edit it, in the present tense. Version history, recurrences and corrections go in
  `CHANGELOG.md`.
- **DO NOT CHASE THE CYRIUS PIN.** cyrius releases **constantly**, so the wrapper being ahead of
  `[package].cyrius` is the **normal, expected state — not drift, not a finding, not a task.** Bump the
  pin only when it is genuinely **necessary** (something the work needs is in the newer toolchain) or
  when the **operator asks for it**. Never bump it "to be current", never open a stack-wide sweep, and
  never report a version difference between this repo and its siblings as a defect — repos legitimately
  sit on different pins. ⚠ A pin bump is a **real source change**, not a number: it re-vendors `lib/`
  and changes the compiled binary, so it is never free housekeeping.

## Documentation

- [`docs/adr/`](docs/adr/) — Architecture Decision Records (*why X over Y?*)
- [`docs/architecture/`](docs/architecture/) — Non-obvious constraints
- [`docs/guides/`](docs/guides/) — Task-oriented how-tos
- [`docs/examples/`](docs/examples/) — Runnable examples
- [`docs/development/state.md`](docs/development/state.md) — Live state
- [`docs/development/roadmap.md`](docs/development/roadmap.md) — Milestones through v1.0

