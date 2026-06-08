#!/bin/bash
# Phase 5d Freeze — Git Commit & Tag Commands
# Execute these commands to freeze and document Phase 5d

set -e

echo "=========================================="
echo "Phase 5d Freeze — Git Operations"
echo "=========================================="

# Stage all Phase 5d changes
echo "[1/5] Staging Phase 5d changes..."
git add -A

# Show what will be committed
echo ""
echo "[2/5] Changes to be committed:"
git diff --cached --stat

# Create freeze commit
echo ""
echo "[3/5] Creating freeze commit..."
git commit -m "Phase 5d Freeze: Causal Visualization Layer

This commit freezes the Phase 5d Causal Visualization Layer with full deterministic guarantees.

New Features:
- Causal backtrace graph with lag-window projection
- Rollback intensity heatmap (per-tick normalization)
- Multi-run delta variance visualization
- Global tick synchronization (unified playback)
- Deterministic SVG/JSON snapshot export
- Real-time determinism badge

Architecture:
- ObservatoryViewModel with frozen projections
- Pure renderer UI (egui, no forensic coupling)
- CognitiveState isolation (UI intent only)
- Forensic core immutable

Guarantees:
✓ All projections computed once at load (never recomputed)
✓ Forensic core immutable from UI interactions
✓ Deterministic rendering (identical input → identical output)
✓ Pure projection semantics (no inference, no hidden state)
✓ Audit-safe snapshot exports

Testing:
✓ Determinism tests pass
✓ Mutation tests pass
✓ Integration tests pass
✓ Performance acceptable

Files Modified:
- src/cognitive/state.rs (new)
- src/cognitive/view_model.rs (new + Phase 5d extensions)
- src/observatory.rs (Phase 5d rendering + export)
- src/main.rs (ViewModel wiring)
- PHASE_5D_FREEZE.md (documentation)

Next: Phase 6 (multi-parameter forensic tuning, rollback chain analysis)"

# Create version tag
echo ""
echo "[4/5] Creating version tag..."
git tag -a v2V.1-Phase5d-FROZEN -m "Phase 5d Release: Deterministic Causal Visualization

Engine Version: 2V.1
Phase: 5d (Causal Visualization Layer)
Status: FROZEN

Key Achievements:
- Backtrace projection deterministic
- Rollback intensity heatmap deterministic
- Multi-run delta visualization deterministic
- Global tick synchronization working
- Snapshot exports reproducible
- GUI fully interactive, zero forensic mutations

Frozen Components:
- Forensic core (Phase 2V)
- Consensus engine (Phase 3b)
- Observatory GUI (Phase 4/5c/5d)
- Cognitive UI layer (Phase 5c)
- Causal visualization (Phase 5d)

All guarantees validated and locked.
Ready for production use.

See PHASE_5D_FREEZE.md for full freeze report."

# Show commit and tag info
echo ""
echo "[5/5] Freeze complete. Summary:"
echo ""
echo "Latest Commit:"
git log -1 --oneline
echo ""
echo "Latest Tags:"
git tag -l -n 3 | head -3
echo ""
echo "=========================================="
echo "Phase 5d is now FROZEN and TAGGED"
echo "Tag: v2V.1-Phase5d-FROZEN"
echo "=========================================="
echo ""
echo "Next Steps:"
echo "1. Review freeze: git show v2V.1-Phase5d-FROZEN"
echo "2. View freeze report: cat PHASE_5D_FREEZE.md"
echo "3. Create release notes from tag"
echo "4. Begin Phase 6 exploratory branch (optional)"
