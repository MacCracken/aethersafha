# Getting started with aethersafha

## Build

```sh
cyrius deps                              # resolve dependencies
cyrius build src/main.cyr build/aethersafha    # compile
cyrius test                              # run tests/*.tcyr
```

## Layout

- `src/main.cyr` — entry point. Top-level `var r = main(); syscall(SYS_EXIT, r);`.
- `tests/` — test suite (`.tcyr` files, auto-discovered by `cyrius test`).
- The original Rust source is not in the tree. Tag `0.17.5` is the last that carries it, at `rust-old/`; read a file with `git show 0.17.5:rust-old/<path>`. A citation `rust-old/<path>[:<line>]` in a comment or doc means that path at that tag.

## Adding a feature

1. Edit `src/main.cyr` (or add a new module and `include` it).
2. Cross-check parity against the record: ADRs 0001–0004, the `.tcyr` suites, and tag `0.17.5`. [`../architecture/002-rust-public-api-map.md`](../architecture/002-rust-public-api-map.md) says where each Rust type went.
3. Add a test case to `tests/aethersafha.tcyr`.
4. Run `cyrius test`.
5. Bump `VERSION` and add a CHANGELOG entry before tagging.

See [`../adr/template.md`](../adr/template.md) when a non-trivial design choice deserves an ADR.
