# Quick Start — 5 Minutes to First Analysis

Get the Deterministic Forensic Observatory running and analyzing kernel traces in 5 minutes.

## Prerequisites

- Bash shell (Linux, macOS, or WSL2 on Windows)
- Git
- 512 MB RAM, 100 MB disk space

## Installation (2 minutes)

```bash
# Clone the repository
git clone https://github.com/yourusername/deterministic-forensic-observatory.git
cd deterministic-forensic-observatory

# Run setup (installs Rust if needed, builds binary)
bash install.sh
```

That's it. The script will:
- ✓ Install Rust (if not present)
- ✓ Build the release binary
- ✓ Run smoke tests
- ✓ Print success message

## First Run: Interactive GUI (2 minutes)

```bash
cargo run --release -- observatory-gui replay.json
```

A window will appear showing:
- **📊 Stats:** Number of runs, divergence clusters, consensus ticks
- **✔ Status:** Determinism badge (green = deterministic)
- **🎚️ Slider:** Scrub through the trace timeline
- **🔴 Clusters:** Click to see divergence regions
- **🔥 Heatmap:** Rollback intensity visualization
- **🔗 Backtrace:** Causal event-to-divergence mapping

**In the GUI:**
- Click "Export SVG Snapshot" to save a forensic report
- Move the slider to scrub through time
- Select a cluster to see details

## Command-Line: Validate a Trace (1 minute)

```bash
cargo run --release -- validate replay.json
```

Output shows:
```
FORENSIC REPORT
Engine: 2V.1
Trace hash: 14269139301645939807
Events: 0
Engine: 2V.1 | Projection: ✓ | Deterministic: ✓ | Attribution: ✓ | Graph: ✓ | Divergence: ✓
```

## Command-Line: Compare Two Runs (< 1 minute)

```bash
# Verify the same trace is deterministic
cargo run --release -- consensus replay.json replay.json
```

Output:
```
Consensus Analysis:
  Runs loaded: 2
  Divergence clusters: 0
  Operator consensus ticks: 10000

✓ No divergences detected (runs are deterministic)
```

If runs **diverge**, you'll see:
```
Consensus Analysis:
  Runs loaded: 2
  Divergence clusters: 3
  Operator consensus ticks: 10000

Cluster Metrics:
  [0] tick 500-520: agreement=0.8234, domains=[Struct]
  [1] tick 1200-1250: agreement=0.7123, domains=[Energy, Memory]
  [2] tick 3000-3010: agreement=0.9456, domains=[Topology]
```

## Command-Line: Export Results (< 1 minute)

```bash
cargo run --release -- consensus-export replay.json replay.json
```

Creates:
- `replay.json.consensus.json` — Structured divergence report
- `replay.json.consensus.csv` — Tabular summary

View the JSON:
```bash
cat replay.json.consensus.json | jq .
```

## Common Workflows

### Workflow 1: Check if a trace is deterministic

```bash
cargo run --release -- validate trace.json
```

If `Divergence: ✓`, the trace is clean.

### Workflow 2: Find where two runs differ

```bash
cargo run --release -- consensus run_a.json run_b.json
cargo run --release -- observatory-gui run_a.json
```

In the GUI, explore the divergence clusters and operator attribution.

### Workflow 3: Batch validate 10 traces

```bash
for trace in traces/*.json; do
  echo "=== $trace ==="
  cargo run --release -- validate "$trace"
done
```

### Workflow 4: Generate forensic report for documentation

```bash
cargo run --release -- observatory-gui trace.json
# [In GUI] Click "Export SVG Snapshot"
# → observatory_tick_000000.svg
```

## File Formats

### Input Trace JSON

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
      "struct_hash": "00000000000000000000000000000001",
      "energy_hash": "00000000000000000000000000000002",
      "topo_hash": "00000000000000000000000000000003",
      "memory_hash": "00000000000000000000000000000004"
    }
  },
  ...
]
```

### Output: consensus.json

```json
{
  "consensus_summary": {
    "runs": 2,
    "total_ticks": 10000,
    "divergence_clusters": 0,
    "avg_cluster_agreement": 1.0000
  },
  "clusters": []
}
```

## Tips & Tricks

### Use the GUI with your own traces

Convert your kernel traces to JSON and load them:

```bash
cargo run --release -- observatory-gui /path/to/my/trace.json
```

### Speed up builds for development

```bash
# Debug build (faster compile, slower runtime)
cargo build
cargo run -- validate trace.json

# Release build (slower compile, faster runtime)
cargo build --release
cargo run --release -- validate trace.json
```

### View detailed diagnostics

```bash
# Enable verbose logging
RUST_LOG=debug cargo run --release -- validate trace.json
```

### Generate multiple snapshots from one trace

```bash
cargo run --release -- observatory-gui trace.json
# [Scrub to tick 0]   → Click "Export SVG Snapshot" → observatory_tick_000000.svg
# [Scrub to tick 100] → Click "Export SVG Snapshot" → observatory_tick_000100.svg
# [Scrub to tick 500] → Click "Export SVG Snapshot" → observatory_tick_000500.svg
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| `error: could not compile` | Run `rustup update` to upgrade Rust |
| `GUI won't launch` | Ensure X11 is available (Linux) or using WSL2 (Windows) |
| `Trace is invalid JSON` | Verify format with `jq . < trace.json` |
| `Out of memory` | Reduce trace size or increase available RAM |

## Next Steps

- Read [README.md](README.md) for full feature list
- See [docs/ARCHITECTURE_DETAILED.md](docs/ARCHITECTURE_DETAILED.md) for complete system design
- Review [docs/PHASE_5D_FREEZE.md](docs/PHASE_5D_FREEZE.md) for formal guarantees

## Questions?

- Check [README.md#FAQ](README.md#faq)
- Open an issue on GitHub
- Email: bigdilly95@gmail.com

---

**You're all set!** Start analyzing kernel traces with:

```bash
cargo run --release -- observatory-gui replay.json
```
