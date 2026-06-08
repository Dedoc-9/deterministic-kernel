//! Forensic analysis: operator attribution, event extraction, causal graphing

use crate::telemetry_reader::ExportTickData;

#[derive(Clone, Debug)]
pub struct ExportReplayResult {
    pub ticks: Vec<ExportTickData>,
    pub trace_hash: String,
}

#[derive(Clone, Copy, Debug)]
pub struct OperatorIntensity {
    pub lie: u8,
    pub diss: u8,
    pub back: u8,
    pub ema: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct OperatorContribution {
    pub lie: f32,
    pub diss: f32,
    pub back: f32,
    pub ema: f32,
}

impl OperatorContribution {
    pub fn normalize(lie: f32, diss: f32, back: f32, ema: f32) -> Self {
        let sum = lie + diss + back + ema;
        if sum < 0.00001 {
            return Self {
                lie: 0.25,
                diss: 0.25,
                back: 0.25,
                ema: 0.25,
            };
        }
        Self {
            lie: lie / sum,
            diss: diss / sum,
            back: back / sum,
            ema: ema / sum,
        }
    }
}

#[derive(Clone, Debug)]
pub enum EventKind {
    ConfidenceDrop,
    EnergyJump,
    LearningSpike,
    ModeTransition,
    Rollback,
    Divergence,
}

#[derive(Clone, Debug)]
pub struct CausalEvent {
    pub tick: usize,
    pub kind: EventKind,
    pub magnitude: f64,
    pub description: String,
}

#[derive(Clone, Debug)]
pub struct EventThresholds {
    pub confidence_drop: u8,
    pub learning_spike: i64,
    pub energy_jump: i64,
}

impl Default for EventThresholds {
    fn default() -> Self {
        Self {
            confidence_drop: 5,
            learning_spike: 1000,
            energy_jump: 5000,
        }
    }
}

/// Derive operator attribution from observable signals
pub fn operator_contribution(
    trace: &[ExportTickData],
    tick: usize,
) -> OperatorContribution {
    let cur = &trace[tick];
    const EPS: f32 = 1.0;

    let learning_norm = (cur.learning_delta.abs() as f32 / 100_000.0).min(1.0);
    let confidence_norm = (100.0 - cur.confidence as f32) / 100.0;
    let rollback_norm = (cur.rollback_count as f32 / 10.0).min(1.0);
    let energy_norm = ((cur.energy_norm.abs() as f32).ln_1p() / 20.0).min(1.0);

    let lie = (learning_norm * 100.0) + EPS;
    let diss = (confidence_norm * 100.0) + EPS;
    let back = (rollback_norm * 100.0) + EPS;
    let ema = (energy_norm * 100.0) + EPS;

    OperatorContribution::normalize(lie, diss, back, ema)
}

/// Extract causal events from trace
pub fn extract_events(
    trace: &[ExportTickData],
    thresholds: &EventThresholds,
) -> Vec<CausalEvent> {
    let mut out = Vec::new();

    for i in 1..trace.len() {
        let prev = &trace[i - 1];
        let cur = &trace[i];

        if prev.confidence > cur.confidence + thresholds.confidence_drop {
            out.push(CausalEvent {
                tick: i,
                kind: EventKind::ConfidenceDrop,
                magnitude: (prev.confidence - cur.confidence) as f64,
                description: format!("Confidence dropped {} → {}", prev.confidence, cur.confidence),
            });
        }

        let de = (cur.energy_norm - prev.energy_norm).abs();
        if de > thresholds.energy_jump {
            out.push(CausalEvent {
                tick: i,
                kind: EventKind::EnergyJump,
                magnitude: de as f64,
                description: format!("Energy jump Δ={}", de),
            });
        }

        let dl = cur.learning_delta.abs();
        if dl > thresholds.learning_spike {
            out.push(CausalEvent {
                tick: i,
                kind: EventKind::LearningSpike,
                magnitude: dl as f64,
                description: format!("Learning spike {}", dl),
            });
        }

        if cur.mode != prev.mode {
            out.push(CausalEvent {
                tick: i,
                kind: EventKind::ModeTransition,
                magnitude: 1.0,
                description: format!("Mode {} → {}", prev.mode, cur.mode),
            });
        }

        if cur.rollback_count > prev.rollback_count {
            out.push(CausalEvent {
                tick: i,
                kind: EventKind::Rollback,
                magnitude: (cur.rollback_count - prev.rollback_count) as f64,
                description: format!("Rollback count {}", cur.rollback_count),
            });
        }
    }

    out
}

#[derive(Clone, Debug)]
pub struct TemporalGraph {
    pub nodes: Vec<EventNode>,
    pub edges: Vec<EventEdge>,
}

#[derive(Clone, Debug)]
pub struct EventNode {
    pub id: usize,
    pub tick: usize,
    pub kind: EventKind,
    pub magnitude: f32,
}

#[derive(Clone, Debug)]
pub struct EventEdge {
    pub from: usize,
    pub to: usize,
    pub strength: f32,
}

impl Default for TemporalGraph {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }
}

pub fn build_temporal_graph(events: &[CausalEvent], max_gap: usize) -> TemporalGraph {
    let mut graph = TemporalGraph::default();

    for (i, ev) in events.iter().enumerate() {
        graph.nodes.push(EventNode {
            id: i,
            tick: ev.tick,
            kind: ev.kind.clone(),
            magnitude: ev.magnitude as f32,
        });
    }

    for i in 0..events.len() {
        for j in (i + 1)..events.len() {
            let dt = events[j].tick - events[i].tick;
            if dt > max_gap {
                break;
            }
            graph.edges.push(EventEdge {
                from: i,
                to: j,
                strength: 1.0 / (dt as f32 + 1.0),
            });
        }
    }

    graph
}
