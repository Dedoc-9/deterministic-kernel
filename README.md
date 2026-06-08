# Deterministic Forensic Observatory — v2V.1

**Production-grade kernel trace analysis system with zero-mutation forensic guarantees.**

Analyze deterministic kernel execution traces, detect multi-run divergences, visualize causal relationships, and export reproducible forensic reports—all without mutating the core analysis.

**Author:** Daniel J. Dillberg   

**Contact:** bigdilly95@gmail.com

## Quick Start

```bash
# Clone and setup (5 minutes)
git clone https://github.com/yourusername/deterministic-forensic-observatory.git
cd deterministic-forensic-observatory
bash install.sh

# Run the observatory
cargo run --release -- observatory-gui replay.json
```

**System Requirements:**
- Rust 1.70+ (installed via `rustup`)
- Linux/macOS/Windows with bash
- 512 MB RAM, 100 MB disk space

## What It Does

### 1. Analyzes Deterministic Kernel Traces
Load frozen kernel execution traces (10,000+ ticks of Q31.32 fixed-point state) and verify determinism across multiple runs.

```bash
# Validate a single trace
cargo run --release -- validate trace.json

# Compare two runs for divergences
cargo run --release -- consensus run1.json run2.json
```

### 2. Detects Divergences with Operator Attribution
Identify where runs diverge and attribute divergence to four operators:
- **LIE:** Learning delta intensity
- **DISS:** Dissipation (confidence loss)
- **BACK:** Backreaction (rollback activity)
- **EMA:** Energy memory lag

```bash
# Export consensus analysis
cargo run --release -- consensus-export run1.json run2.json
```

### 3. Visualizes Causality Interactively
Launch a windowed GUI with:
- **Causal backtrace graph** (event → divergence mapping)
- **Rollback intensity heatmap** (per-tick color gradient)
- **Multi-run delta visualization** (cross-run variance)
- **Global tick scrubber** (synchronized playback)
- **Real-time determinism badge** (✔/✖ validation)

```bash
cargo run --release -- observatory-gui trace.json
```

### 4. Exports Reproducible Forensic Reports
Generate audit-safe snapshots in multiple formats:

```bash
# SVG snapshots (structural)
cargo run --release -- observatory-gui trace.json
# [In GUI] Click "Export SVG Snapshot"

# JSON/CSV reports
cargo run --release -- consensus-export run1.json run2.json
# → run1.json.consensus.json, run1.json.consensus.csv

# Forensic validation reports
cargo run --release -- forensic-export trace.json
# → trace.json.forensic.json, trace.json.forensic.csv
```

## Key Guarantees

| Guarantee | Proof | Implication |
|-----------|-------|-------------|
| **Zero Mutation** | Forensic core is immutable; UI only reads | Same trace → identical analysis |
| **Deterministic** | No RNG, no floating-point drift | Identical input → identical output |
| **Reproducible** | Snapshots are bitwise identical on re-export | Audit-safe forensic reports |
| **Hash Consistent** | Trace hash unchanged before/after UI | Structural integrity verified |

## Architecture

```
Kernel Trace (frozen)
    ↓
Consensus Engine (deterministic divergence detection)
    ├─ Multi-run comparison
    ├─ Operator attribution (LIE/DISS/BACK/EMA)
    ├─ Cluster merging (contiguous divergence regions)
    └─ Agreement strength metrics
    ↓
ViewModel (pure projection layer)
    ├─ Backtrace edges (deterministic lag-window)
    ├─ Rollback intensity (per-tick normalization)
    ├─ Multi-run delta (cross-run variance)
    └─ CognitiveState (UI intent only)
    ↓
Observatory GUI (pure renderer, zero forensic logic)
    ├─ Causal backtrace visualization
    ├─ Heatmap rendering
    ├─ Cluster selection & detail view
    └─ Snapshot export (SVG/JSON)
```

**No feedback loop.** UI cannot alter analysis.

## Installation

See [INSTALL.md](INSTALL.md) or run:

```bash
bash install.sh
```

This installs Rust, clones the repo, builds the observatory, and runs smoke tests.

## Usage

### Command-Line Interface

#### Validate a trace
```bash
cargo run --release -- validate replay.json
```
Output: Validation report with forensic status

#### Compare two runs
```bash
cargo run --release -- consensus run_a.json run_b.json
```
Output: Divergence clusters, agreement metrics

#### Export consensus analysis
```bash
cargo run --release -- consensus-export run_a.json run_b.json
```
Output:
- `run_a.json.consensus.json` — Structured report
- `run_a.json.consensus.csv` — Tabular export

#### Export forensic report
```bash
cargo run --release -- forensic-export trace.json
```
Output:
- `trace.json.forensic.json` — Validation flags, event counts
- `trace.json.forensic.csv` — Single-row summary

### Interactive Observatory (GUI)

```bash
cargo run --release -- observatory-gui trace.json
```

**Features:**
- 📊 Run count, divergence cluster count, consensus ticks
- 🎚️ Global tick slider (controls all views)
- 🔴 Divergence cluster viewport (scrollable, selectable)
- 🔥 Rollback intensity heatmap (color gradient)
- 🔗 Causal backtrace graph (event → divergence edges)
- 📍 Cluster detail panel (with cross-run variance)
- 📸 Snapshot export (SVG, JSON)

## Examples

### Single-Run Determinism Check
```bash
# Load identical runs → expect zero divergence clusters
cargo run --release -- consensus trace.json trace.json
# Output: 0 divergence clusters, agreement = 1.0000 ✓
```

### Multi-Run Divergence Analysis
```bash
# Compare two different kernel runs
cargo run --release -- consensus-export run_optimized.json run_baseline.json

# View results in GUI
cargo run --release -- observatory-gui run_optimized.json

# [In GUI]
# - Explore divergence clusters by tick range
# - Inspect operator attribution per cluster
# - Export snapshot for documentation
```

### Batch Processing
```bash
# Validate 10 traces
for trace in traces/*.json; do
  echo "Validating $trace..."
  cargo run --release -- validate "$trace"
done
```

## Testing

Run the smoke test suite:

```bash
cargo test --release --lib
```

Expected output:
```
running 5 tests
test forensic_smoke::test_stable_equilibrium_loads ... ok
test forensic_smoke::test_consensus_deterministic ... ok
test forensic_smoke::test_divergence_detection ... ok
test forensic_smoke::test_export_reproducible ... ok
test forensic_smoke::test_gui_launch ... ok

test result: ok. 5 passed
```

## Documentation

- **[INSTALL.md](INSTALL.md)** — Detailed installation guide
- **[QUICKSTART.md](QUICKSTART.md)** — 5-minute tutorial
- **[PHASE_5D_FREEZE.md](PHASE_5D_FREEZE.md)** — Complete freeze report with guarantees
- **[docs/ARCHITECTURE_DETAILED.md](docs/ARCHITECTURE_DETAILED.md)** — Complete system architecture (phases 2Vb–5d)
- **[src/](src/)** — Annotated source code with comments

## Building from Source

```bash
# Build release binary (optimized, ~3-5 seconds)
cargo build --release

# Binary location
./target/release/deterministic-launcher

# Or run directly
cargo run --release -- --help
```

## Performance

| Operation | Time | Memory |
|-----------|------|--------|
| Load 10K-tick trace | <50ms | 5 MB |
| Consensus analysis (2 runs) | <100ms | 10 MB |
| GUI startup | <200ms | 15 MB |
| Snapshot export | <10ms | - |

## FAQ

### Q: Can the GUI modify traces?
**A:** No. The GUI is a pure renderer. It reads consensus data but cannot write back or recompute analysis. All mutations are isolated to UI state (tick selection, cluster selection).

### Q: How do I verify traces are deterministic?
**A:** Load the same trace twice:
```bash
cargo run --release -- consensus trace.json trace.json
```
If clusters = 0 and agreement = 1.0000, the trace is deterministic.

### Q: What formats do traces use?
**A:** Traces are JSON arrays of tick objects:
```json
[
  {
    "tick": 0,
    "mode": 0,
    "confidence": 100,
    "energy_norm": 1000000,
    "rollback_count": 0,
    "learning_delta": 0,
    "hashes": {
      "struct_hash": "...",
      "energy_hash": "...",
      "topo_hash": "...",
      "memory_hash": "..."
    }
  },
  ...
]
```

### Q: Can I use this in production?
**A:** Yes. The system is frozen, deterministic, and fully validated. See [PHASE_5D_FREEZE.md](PHASE_5D_FREEZE.md) for formal guarantees.

### Q: How do I contribute?
**A:** See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

AGPL-3.0. See [LICENSE](LICENSE).

## Citation

```bibtex
@software{dillberg2026observatory,
  title={Deterministic Forensic Observatory v2V.1},
  author={Dillberg, Daniel J.},
  year={2026},
  url={https://github.com/yourusername/deterministic-forensic-observatory}
}
```

## Support

- **Issues:** GitHub Issues
- **Discussions:** GitHub Discussions
- **Email:** bigdilly95@gmail.com

---

**Status:** ✓ Production Ready (v2V.1-Phase5d-FROZEN)  
**Last Updated:** 2026-06-08  
**Build:** `cargo build --release`
