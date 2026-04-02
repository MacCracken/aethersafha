# Development Roadmap

## v0.1.0 — Initial Extraction (current)

- [x] Extract from agnosticos/userland/desktop-environment
- [x] Standalone Cargo.toml with path deps to agnostik + agnosys
- [x] All existing source, benchmarks, and tests carried over
- [ ] Verify `cargo check --all-features` passes standalone
- [ ] Verify `cargo test --all-features` passes standalone
- [ ] First benchmark baseline via `./scripts/bench-history.sh`

## v0.2.0 — Standalone Hardening

- [ ] P(-1) scaffold hardening pass
- [ ] CI workflows (ci.yml, release.yml)
- [ ] Integration tests in `tests/`
- [ ] Example in `examples/`
- [ ] Full clippy + fmt + audit + deny clean

## v0.3.0 — Independent Build

- [ ] Replace agnostik path dep with crates.io dep (when agnostik publishes)
- [ ] Replace agnosys path dep with crates.io dep (when agnosys publishes)
- [ ] Own Cargo.lock, independent of agnosticos workspace

## v1.0.0 Criteria

- [ ] All protocol handlers production-tested
- [ ] Plugin host API stable
- [ ] Screen capture/recording API stable
- [ ] Accessibility API stable
- [ ] 80%+ code coverage
- [ ] Independent ark package: `ark upgrade aethersafha`
