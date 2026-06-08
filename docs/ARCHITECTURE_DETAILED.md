# Deterministic Forensic Observatory — Complete System Architecture

**Engine Version:** 2V.1  
**Status:** ✓ FULLY FROZEN (Phase 2Vb → 5d)  
**Date:** 2026-06-08  
**Build:** Release (optimized)  

---

## Executive Summary

The Deterministic Forensic Observatory is a read-only kernel trace analysis system that:

1. **Loads immutable forensic traces** (Q31.32 fixed-point kernel execution)
2. **Detects divergences** (multi-run consensus analysis with operator attribution)
3. **Visualizes causality** (backtrace projection, rollback heatmap, variance overlays)
4. **Exports deterministically** (JSON/CSV/SVG, audit-safe, reproducible)
5. **Maintains zero mutations** (forensic core untouched by UI)

**Key Guarantee:** Same input trace → identical output visualization → reproducible audit trail.

---

## Phase Breakdown

### Phase 2Vb: Forensic Export Layer ✓ FROZEN

**Purpose:** Serialize forensic analysis to portable formats

**Components:**
- `src/forensic/report.rs` — Report generation (engine version, validation flags, event counts)
- `src/forensic/mod.rs` — Module structure (assertions, fixtures, validation, report)
- CLI: `forensic-export <trace.json>` → `.forensic.json`, `.forensic.csv`

**Guarantees:**
- ✓ Pure projection (no recomputation)
- ✓ Hash consistency (trace hash unchanged before/after)
- ✓ Deterministic serialization
- ✓ Validation semantics untouched

**Files Generated:**
```
replay.json.forensic.json  — Report with validation flags, event count, hash
replay.json.forensic.csv   — Tabular export (1 row per report)
```

---

### Phase 3b: Consensus Engine ✓ FROZEN

**Purpose:** Multi-run divergence detection with operator attribution

**Components:**
- `src/forensic/consensus.rs` — Consensus analysis (cluster merging, agreement metrics)
- `src/forensic/validation.rs` — Validation gates (projection purity, determinism, attribution)
- CLI: `consensus <run1.json> <run2.json>` → prints cluster summary

**Key Structures:**
```rust
DivergenceCluster {
    start_tick: usize,
    end_tick: usize,
    domains: Vec<DivergenceDomain>,         // Struct, Energy, Topology, Memory
    operator_intensity: Vec<f32>,           // [LIE, DISS, BACK, EMA] averages
}

OperatorConsensus {
    lie_avg: f32,       // Learning delta intensity
    diss_avg: f32,      // Dissipation (confidence loss)
    back_avg: f32,      // Backreaction (rollback activity)
    ema_avg: f32,       // Energy memory lag
}

ClusterMetrics {
    avg_agreement: f32, // Variance-inverse metric [0,1]
    tick_count: usize,
}
```

**Guarantees:**
- ✓ Pairwise divergence detection (runs[0] vs runs[1])
- ✓ Contiguous cluster merging (same domain)
- ✓ Operator consensus aggregation
- ✓ Agreement strength metric (deterministic)
- ✓ Zero recomputation in UI

**Files Generated:**
```
replay.json.consensus.json  — Clusters + agreement metrics
replay.json.consensus.csv   — Tabular cluster summary
```

---

### Phase 4/5b: Observatory GUI (Windowed) ✓ FROZEN

**Purpose:** Interactive visualization of forensic data without mutation

**Components:**
- `src/observatory.rs` — egui-based GUI app
- `src/cognitive/state.rs` — UI-only state (CognitiveState, GlobalPlayback)
- `src/cognitive/view_model.rs` — ViewModel layer (read-only access bridge)
- CLI: `observatory-gui <trace.json>` → launches 1000×700 window

**Key Rendering:**
- Determinism badge (✔/✖)
- Cluster viewport (scrollable, selectable)
- Operator intensity heatmap (per-cluster, per-operator)
- Causal backtrace placeholder

**Guarantees:**
- ✓ Pure renderer (no forensic logic)
- ✓ Zero mutation of consensus data
- ✓ Responsive UI (sub-100ms frame time)
- ✓ Cluster selection (UI state only)

---

### Phase 5c: Cognitive UI Layer ✓ FROZEN

**Purpose:** Separation of forensic truth from interactive cognition

**Architecture:**
```
ObservatoryViewModel {
    analysis: ConsensusAnalysis,        // Frozen forensic data
    state: CognitiveState,              // Mutable UI intent only
}

CognitiveState {
    selected_run: usize,
    selected_cluster: Option<usize>,
    focus_mode: FocusMode,              // All | ClusterOnly | DivergenceOnly
    comparison_mode: bool,
    global: GlobalPlayback {
        tick: usize,
        min: usize,
        max: usize,
    }
}
```

**Key Feature:**
- **Global Tick Synchronization:** Single slider controls all views
- **Unified Playback:** All panels bind to `global.tick`
- **No Recomputation:** ViewModel projection cached at construction

**Guarantees:**
- ✓ ViewModel is sole mutable boundary
- ✓ Forensic data never touched
- ✓ UI state never affects consensus
- ✓ Deterministic binding (same tick → same view)

---

### Phase 5d: Causal Visualization Layer ✓ FROZEN

**Purpose:** Deterministic causal analysis projections (backtrace, rollback, variance)

**New ViewModel Projections:**

#### 1. Backtrace Edges
```rust
backtrace_edges: Vec<BacktraceEdge> {
    source_tick: usize,
    target_tick: usize,
    strength: f32,          // 1.0 / (dt + 1)
}
```
**Computation:** For each cluster, event ticks in [cluster_start-10, cluster_start) → cluster_start with decay strength  
**Guarantee:** Deterministic lag-window, no inference model

#### 2. Rollback Intensity
```rust
rollback_intensity: Vec<f32>  // per-tick max rollback / 10.0, clamped [0,1]
```
**Computation:** `max(rollback_count across runs) / 10.0`  
**Guarantee:** Normalized, deterministic, visual encoding only

#### 3. Multi-run Delta
```rust
multi_run_delta: Vec<f32>  // per-tick cross-run energy variance
```
**Computation:** `sqrt(variance) / (mean + ε)` for energy values across runs  
**Guarantee:** Statistical projection, no inference

**UI Rendering:**
- Rollback heatmap: 100-tick sample, color gradient
- Backtrace list: 10 edges per window, filtered by tick ±50
- Delta display: Max variance in selected cluster detail
- Snapshot export: SVG/JSON deterministic

**Guarantees:**
- ✓ All projections computed once at ViewModel construction
- ✓ Never recomputed during interactive UI use
- ✓ Deterministic rendering (seed-independent)
- ✓ Pure projection semantics (read-only)
- ✓ Audit-safe snapshot exports

---

## Data Flow Architecture

```
┌─────────────────────────────────────────┐
│     Replay.json (frozen trace)          │
│  (10,000 ticks × 7 fields per tick)    │
└────────────────┬────────────────────────┘
                 │
                 ↓ telemetry_reader::load()
        ┌─────────────────────┐
        │ Vec<ExportTickData> │
        │  (immutable)        │
        └────────┬────────────┘
                 │
    ┌────────────┴────────────┐
    │                         │
    ↓                         ↓
┌──────────────┐      ┌──────────────┐
│  Run A       │      │  Run B       │
│  Ticks[n]   │      │  Ticks[n]    │
└──────┬───────┘      └───────┬──────┘
       │                      │
       └──────────┬───────────┘
                  ↓ ConsensusAnalysis::new(vec![A, B])
         ┌────────────────────────────┐
         │ ConsensusAnalysis (frozen) │
         ├────────────────────────────┤
         │ runs: [A, B]               │
         │ divergence_clusters: [...] │
         │ operator_consensus: [...]  │
         │ cluster_metrics: [...]     │
         └────────┬───────────────────┘
                  │
                  ↓ ObservatoryViewModel::new()
         ┌────────────────────────────┐
         │ ViewModel (construct-time) │
         ├────────────────────────────┤
         │ analysis: frozen (read)    │
         │ backtrace_edges: compute() │
         │ rollback_intensity: comp() │
         │ multi_run_delta: compute() │
         │ state: CognitiveState      │
         └────────┬───────────────────┘
                  │
                  ↓ eframe::run_native()
         ┌────────────────────────────┐
         │ ObservatoryApp (pure UI)   │
         ├────────────────────────────┤
         │ read: ViewModel.analysis   │
         │ mutate: ViewModel.state    │
         │ render: egui panels        │
         │ export: SVG/JSON           │
         └────────────────────────────┘
```

**Key Property:** No feedback loop. UI cannot alter analysis.

---

## Deterministic Guarantees

### 1. Hash Consistency
```
trace_hash_before = HASH(ticks)
[UI interactions]
trace_hash_after = HASH(ticks)
assert_eq!(trace_hash_before, trace_hash_after)  // Always true
```

### 2. Operator Consensus Determinism
```
attribution[0] = operator_contribution(run, tick)  // Deterministic function
attribution[1] = operator_contribution(run, tick)  // Same output
assert_eq!(attribution[0], attribution[1])  // Always true
```

### 3. Backtrace Projection Determinism
```
backtrace_0 = compute_backtrace(consensus)
backtrace_1 = compute_backtrace(consensus)
assert_eq!(backtrace_0, backtrace_1)  // Always true
```

### 4. UI Rendering Determinism
```
frame_0 = render(vm, tick)
[no changes to vm]
frame_1 = render(vm, tick)
assert_eq!(frame_0, frame_1)  // Always true (visually identical)
```

### 5. Snapshot Export Determinism
```
svg_0 = export_svg(vm, tick)
[reload VM from same data]
svg_1 = export_svg(vm, tick)
assert_eq!(svg_0, svg_1)  // Bitwise identical
```

---

## CLI Commands

### Forensic Validation
```bash
cargo run --release -- validate <trace.json>
```
Output: Validation report (projection purity, determinism, attribution)

### Consensus Analysis
```bash
cargo run --release -- consensus <run1.json> <run2.json>
```
Output: Consensus summary (clusters, agreement metrics)

### Consensus Export
```bash
cargo run --release -- consensus-export <run1.json> <run2.json>
```
Output: `run1.json.consensus.json`, `run1.json.consensus.csv`

### Forensic Export
```bash
cargo run --release -- forensic-export <trace.json>
```
Output: `trace.json.forensic.json`, `trace.json.forensic.csv`

### Observatory GUI (Interactive)
```bash
cargo run --release -- observatory-gui <trace.json>
```
Output: Windowed GUI (1000×700) with causal visualization

---

## Test Coverage

### Determinism Tests
- [x] Identical runs → zero divergence clusters
- [x] Operator consensus reproducible
- [x] Backtrace edges deterministic
- [x] Rollback intensity stable
- [x] Delta heatmap consistent

### Mutation Tests
- [x] Forensic data read-only during UI interaction
- [x] Consensus hash unchanged after UI operations
- [x] ViewModel state mutable, analysis frozen

### Integration Tests
- [x] GUI launch with single trace
- [x] Tick scrub updates all views
- [x] Cluster selection updates detail panel
- [x] Snapshot exports create files

### Performance Tests
- [x] ViewModel construction: <100ms
- [x] Frame render: <100ms
- [x] Heatmap sampling: real-time (no lag)
- [x] No memory leaks (egui cleanup verified)

---

## Files & Metrics

```
Source Files:
  src/forensic/assertions.rs      —  30 lines (validation helpers)
  src/forensic/fixtures.rs        — 403 lines (synthetic traces)
  src/forensic/validation.rs      — 155 lines (validation gates)
  src/forensic/report.rs          —  89 lines (report generation + export)
  src/forensic/consensus.rs       — 250 lines (cluster merging, metrics)
  src/forensic/mod.rs             —   7 lines (module structure)
  src/forensics.rs                — 200 lines (operator contribution, events)
  src/cognitive/state.rs          —  40 lines (UI state structures)
  src/cognitive/view_model.rs     — 140 lines (frozen projections)
  src/cognitive/mod.rs            —   3 lines (module structure)
  src/observatory.rs              — 280 lines (UI rendering + export)
  src/main.rs                     — 240 lines (CLI + ViewModel wiring)
  src/lib.rs                      —  10 lines (crate exports)

Total: ~1,850 lines of Rust code

Dependencies:
  serde, serde_json  — serialization
  sha2               — hashing
  egui, eframe       — GUI
  crossterm, ratatui — TUI (from Phase 1, reusable)

Build:
  cargo build --release  → ~3-5 seconds
  Binary size: ~15 MB (release)
```

---

## Known Limitations & Future Work

### Current Limitations (by design)
1. **Backtrace lag-window:** Fixed 10-tick window (configurable in Phase 5e)
2. **Pairwise divergence:** Compares runs[0] vs runs[1] only; N-way in Phase 6
3. **SVG placeholder:** Snapshot content is structural, not graphical (Phase 6)
4. **Focus mode skeleton:** UI filters only; filtering at consensus level in Phase 5e
5. **No animation:** Tick scrub is instant; animation framework deferred

### Future Phases

**Phase 6:** Multi-parameter forensic tuning
- Configurable lag-window (5-50 ticks)
- N-way divergence analysis (run A vs B vs C...)
- Rollback chain intensity (visual retry patterns)
- Operator sensitivity analysis

**Phase 7:** Advanced visualization
- Full SVG/PNG rendering (backtrace graph, cluster heatmaps)
- Animated divergence propagation
- Multi-run overlay comparisons
- 3D phase-space visualization

**Phase 8:** Real-time kernel integration
- Live trace streaming from kernel
- Continuous consensus monitoring
- Alert system for divergence detection

---

## Sign-Off

| Criterion | Status | Notes |
|-----------|--------|-------|
| Forensic Core | ✓ Frozen | 2V.1, immutable, hash-consistent |
| Consensus Engine | ✓ Frozen | Multi-run, deterministic |
| Export Layer | ✓ Frozen | JSON/CSV/SVG, reproducible |
| Observatory GUI | ✓ Frozen | egui, interactive, zero mutations |
| Cognitive Layer | ✓ Frozen | ViewModel separation, UI-only state |
| Causal Visualization | ✓ Frozen | Backtrace, rollback, delta projections |
| Determinism | ✓ Verified | All tests pass |
| Audit Safety | ✓ Verified | Snapshots reproducible |
| Performance | ✓ Acceptable | Sub-100ms frame time |
| Documentation | ✓ Complete | Freeze report + this summary |

**System Status:** ✓ PRODUCTION READY

---

**Release:** v2V.1-Phase5d-FROZEN  
**Date Frozen:** 2026-06-08  
**Next Review:** Phase 6 exploratory branch
