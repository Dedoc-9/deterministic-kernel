//! ViewModel layer — bridge between immutable forensic core and mutable UI state
//! Phase 5d: Extended with causal visualization projections (pure, read-only)

use crate::forensic::consensus::ConsensusAnalysis;
use super::state::{CognitiveState, GlobalPlayback};

/// Phase 5d: Causal projection edges (deterministic, read-only)
#[derive(Clone, Debug)]
pub struct BacktraceEdge {
    pub source_tick: usize,
    pub target_tick: usize,
    pub strength: f32,
}

/// ViewModel: controlled, read-only access to forensic data + mutable UI state
/// CRITICAL INVARIANT: forensic data is never mutated or recomputed from UI
#[derive(Clone)]
pub struct ObservatoryViewModel {
    pub analysis: ConsensusAnalysis,
    pub state: CognitiveState,

    // Phase 5d: Pure projection layers (computed once, never mutated by UI)
    pub backtrace_edges: Vec<BacktraceEdge>,
    pub rollback_intensity: Vec<f32>,     // per tick, [0,1]
    pub multi_run_delta: Vec<f32>,        // cross-run variance per tick
}

impl ObservatoryViewModel {
    /// Construct ViewModel from frozen ConsensusAnalysis
    /// Phase 5d: Pre-compute all causal projections (deterministic, one-time)
    pub fn new(analysis: ConsensusAnalysis) -> Self {
        let max_tick = analysis.operator_consensus.len().saturating_sub(1);

        let mut vm = Self {
            analysis,
            state: CognitiveState {
                global: GlobalPlayback {
                    tick: 0,
                    min: 0,
                    max: max_tick,
                },
                ..Default::default()
            },
            backtrace_edges: Vec::new(),
            rollback_intensity: Vec::new(),
            multi_run_delta: Vec::new(),
        };

        // Phase 5d: Deterministic projection computation (happens once, never recomputed)
        vm.compute_backtrace();
        vm.compute_rollback_intensity();
        vm.compute_multi_run_delta();

        vm
    }

    /// Phase 5d: Build causal backtrace projection (pure, deterministic)
    /// Maps event ticks to divergence cluster ticks with lag window
    fn compute_backtrace(&mut self) {
        self.backtrace_edges.clear();

        for cluster in &self.analysis.divergence_clusters {
            let cluster_start = cluster.start_tick;

            // Deterministic projection: events within 10-tick lag window map to cluster start
            for source_tick in cluster_start.saturating_sub(10)..cluster_start {
                if source_tick < self.analysis.operator_consensus.len() {
                    let strength = 1.0 / (cluster_start - source_tick + 1) as f32;
                    self.backtrace_edges.push(BacktraceEdge {
                        source_tick,
                        target_tick: cluster_start,
                        strength,
                    });
                }
            }
        }
    }

    /// Phase 5d: Compute rollback intensity heatmap (pure projection)
    /// Normalizes rollback_count per tick for visual encoding
    fn compute_rollback_intensity(&mut self) {
        self.rollback_intensity = (0..self.analysis.operator_consensus.len())
            .map(|tick| {
                // Find max rollback in any run at this tick
                let max_rollback = self.analysis.runs.iter()
                    .filter_map(|run| run.get(tick))
                    .map(|tick_data| tick_data.rollback_count as f32)
                    .fold(0.0f32, f32::max);

                (max_rollback / 10.0).min(1.0)
            })
            .collect();
    }

    /// Phase 5d: Compute cross-run delta heatmap (pure projection)
    /// Measures variance in operator attribution across runs
    fn compute_multi_run_delta(&mut self) {
        self.multi_run_delta = (0..self.analysis.operator_consensus.len())
            .map(|tick| {
                if self.analysis.runs.is_empty() || self.analysis.runs[0].is_empty() {
                    return 0.0;
                }

                // Collect energy values from all runs at this tick
                let energy_vals: Vec<f32> = self.analysis.runs.iter()
                    .filter_map(|run| run.get(tick))
                    .map(|tick_data| tick_data.energy_norm as f32)
                    .collect();

                if energy_vals.is_empty() {
                    return 0.0;
                }

                // Compute normalized variance
                let mean = energy_vals.iter().sum::<f32>() / energy_vals.len() as f32;
                let variance = energy_vals.iter()
                    .map(|&e| (e - mean).powi(2))
                    .sum::<f32>() / energy_vals.len() as f32;

                (variance.sqrt() / (mean + 1e-6)).min(1.0)
            })
            .collect();
    }

    /// Pure view filter — reads only, no mutation
    pub fn active_clusters(&self) -> &[crate::forensic::consensus::DivergenceCluster] {
        &self.analysis.divergence_clusters
    }

    /// Get current playback tick
    pub fn current_tick(&self) -> usize {
        self.state.global.tick.min(self.state.global.max)
    }

    /// Mutate UI state only (safe, does not touch forensic data)
    pub fn set_tick(&mut self, tick: usize) {
        self.state.global.tick = tick.min(self.state.global.max);
    }

    /// Set selected cluster (UI state only)
    pub fn set_selected_cluster(&mut self, cluster_idx: Option<usize>) {
        self.state.selected_cluster = cluster_idx;
    }

    /// Get consensus tick at current playback position
    pub fn consensus_at_tick(&self) -> Option<&crate::forensic::consensus::OperatorConsensus> {
        let tick = self.current_tick();
        if tick < self.analysis.operator_consensus.len() {
            Some(&self.analysis.operator_consensus[tick])
        } else {
            None
        }
    }

    /// Get run count (read-only from forensic core)
    pub fn run_count(&self) -> usize {
        self.analysis.runs.len()
    }

    /// Get total consensus ticks (read-only from forensic core)
    pub fn total_consensus_ticks(&self) -> usize {
        self.analysis.operator_consensus.len()
    }

    /// Get divergence cluster count (read-only from forensic core)
    pub fn divergence_cluster_count(&self) -> usize {
        self.analysis.divergence_clusters.len()
    }
}
