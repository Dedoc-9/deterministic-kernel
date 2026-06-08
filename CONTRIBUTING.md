# Contributing

Thank you for your interest in the Deterministic Forensic Observatory! This document outlines how to contribute.

## Code of Conduct

- Be respectful and professional
- Assume good intentions
- Focus on code quality, not ego
- Help others learn

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork:**
   ```bash
   git clone https://github.com/yourusername/deterministic-forensic-observatory.git
   cd deterministic-forensic-observatory
   ```
3. **Create a feature branch:**
   ```bash
   git checkout -b feature/your-feature-name
   ```
4. **Build and test:**
   ```bash
   cargo build
   cargo test --lib
   ```

## Development Workflow

### Before You Start

- Check existing issues and pull requests
- Open a GitHub issue to discuss major changes
- Keep changes focused and scoped

### Code Style

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Run tests
cargo test --release

# Check documentation
cargo doc --open
```

### Commit Messages

Write clear, descriptive commits:

```
Fix: operator consensus calculation for energy divergence

- Correct normalization factor in rollback_intensity
- Add test case for multi-run agreement metric
- Ensure backward compatibility with existing reports

Fixes #123
```

Use conventional prefixes:
- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation
- `test:` Test additions
- `refactor:` Code restructuring
- `perf:` Performance improvement

### Testing

All contributions must include tests:

```bash
# Add test to tests/forensic_smoke.rs
#[test]
fn test_my_feature() {
    let analysis = ConsensusAnalysis::new(...);
    assert_eq!(analysis.some_property(), expected_value);
}

# Run tests
cargo test --release
```

## Areas for Contribution

### Phase 5e: Advanced UI Features

- [ ] Configurable lag-window (5-50 ticks)
- [ ] Focus mode clustering (ClusterOnly, DivergenceOnly filters)
- [ ] Hover tooltips with per-tick operator details
- [ ] Jump-to-tick search (find max divergence, max rollback)

**File:** `src/observatory.rs`  
**Difficulty:** Medium

### Phase 6: Multi-run Analysis

- [ ] N-way divergence (A vs B vs C...)
- [ ] Operator sensitivity analysis
- [ ] Rollback chain visualization
- [ ] Multi-parameter tuning

**File:** `src/forensic/consensus.rs`  
**Difficulty:** Hard

### Phase 7: Advanced Visualization

- [ ] Full SVG/PNG rendering (backtrace graph, cluster heatmaps)
- [ ] Animated divergence propagation
- [ ] 3D phase-space visualization
- [ ] Export to PDF/HTML

**File:** `src/observatory.rs`  
**Difficulty:** Hard

### Quality of Life

- [ ] Batch processing CLI
- [ ] Trace file validation/conversion tools
- [ ] Performance profiling & optimization
- [ ] Documentation improvements

**File:** Any  
**Difficulty:** Easy-Medium

## Testing Requirements

All changes must pass:

```bash
# Format check
cargo fmt -- --check

# Lint check
cargo clippy

# Test suite
cargo test --release

# Build check
cargo build --release
```

## Documentation

- Update [README.md](README.md) for new features
- Add doc comments to public functions
- Update [docs/ARCHITECTURE_DETAILED.md](docs/ARCHITECTURE_DETAILED.md) for architectural changes
- Include examples in pull request description

## Pull Request Process

1. **Ensure tests pass:**
   ```bash
   cargo test --release
   cargo fmt
   cargo clippy
   ```

2. **Update documentation** if necessary

3. **Create pull request** with:
   - Clear title and description
   - Link to related issues
   - Test results
   - Screenshots (for UI changes)

4. **Wait for review** (typically 24-48 hours)

5. **Address feedback** and re-request review

6. **Merge** when approved

## Release Process

Releases are tagged as `v2V.1-Phase5x-FROZEN`:

```bash
# Create release tag
git tag -a v2V.1-Phase6-FROZEN -m "Phase 6: Multi-run analysis"

# Push tag
git push origin v2V.1-Phase6-FROZEN
```

Releases include:
- Changelog (based on commits)
- Updated documentation
- Binary for each platform

## Architecture Constraints

**DO NOT VIOLATE THESE INVARIANTS:**

1. **Forensic core immutability** — ConsensusAnalysis must never be modified from UI
2. **Deterministic rendering** — Same input → identical output (no RNG, no floating-point drift)
3. **Pure projection semantics** — All UI projections are read-only transforms
4. **Hash consistency** — Trace hash must remain unchanged before/after analysis

If you need to change these, open a GitHub issue for discussion first.

## Performance Guidelines

- Keep frame render time < 100ms (60 FPS)
- Precompute projections at load time, not per-frame
- Avoid allocations in hot paths
- Profile with `cargo flamegraph` if unsure

```bash
# Install flamegraph
cargo install flamegraph

# Profile a command
cargo flamegraph --release -- validate trace.json
# View graph: flamegraph.svg
```

## Debug Tips

```bash
# Enable logging
RUST_LOG=debug cargo run --release -- validate trace.json

# Use debugger (with gdb or lldb)
rust-gdb target/release/deterministic-launcher

# Inspect trace file
jq '.[:10]' replay.json  # View first 10 ticks

# Profile memory
valgrind ./target/release/deterministic-launcher
```

## Helpful Resources

- **Rust Book:** https://doc.rust-lang.org/book/
- **Rust API Guidelines:** https://rust-lang.github.io/api-guidelines/
- **egui Docs:** https://docs.rs/egui/
- **Determinism Best Practices:** See PHASE_5D_FREEZE.md

## Questions?

- Open a GitHub Discussion
- Comment on an issue
- Email: bigdilly95@gmail.com

## License

By contributing, you agree that your contributions will be licensed under AGPL-3.0 (same as the project).

---

**Thank you for contributing!** 🙏
