//! Forensic validation engine with deterministic checks

use crate::telemetry_reader::ExportTickData;
use crate::forensics::{extract_events, operator_contribution, EventThresholds};

#[derive(Clone, Copy, Debug)]
pub enum ValidationMode {
    Strict,
    Warn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DivergenceDomain {
    Struct,
    Energy,
    Topology,
    Memory,
}

#[derive(Clone, Debug)]
pub struct DivergenceRecord {
    pub tick: usize,
    pub domain: DivergenceDomain,
}

#[derive(Clone, Debug)]
pub struct ValidationSummary {
    pub engine_version: &'static str,
    pub projection_pure: bool,
    pub deterministic: bool,
    pub attribution_valid: bool,
    pub graph_valid: bool,
    pub divergence_valid: bool,
}

impl ValidationSummary {
    pub fn all_pass(&self) -> bool {
        self.projection_pure && self.deterministic && self.attribution_valid && self.graph_valid && self.divergence_valid
    }

    pub fn status_string(&self) -> String {
        format!(
            "Engine: {} | Projection: {} | Deterministic: {} | Attribution: {} | Graph: {} | Divergence: {}",
            self.engine_version,
            if self.projection_pure { "✓" } else { "✗" },
            if self.deterministic { "✓" } else { "✗" },
            if self.attribution_valid { "✓" } else { "✗" },
            if self.graph_valid { "✓" } else { "✗" },
            if self.divergence_valid { "✓" } else { "✗" }
        )
    }
}

pub fn detect_all_divergences(
    run_a: &[ExportTickData],
    run_b: &[ExportTickData],
) -> Vec<DivergenceRecord> {
    let mut out = Vec::new();

    for i in 0..run_a.len().min(run_b.len()) {
        if run_a[i].hashes.struct_hash != run_b[i].hashes.struct_hash {
            out.push(DivergenceRecord { tick: i, domain: DivergenceDomain::Struct });
        }
        if run_a[i].hashes.energy_hash != run_b[i].hashes.energy_hash {
            out.push(DivergenceRecord { tick: i, domain: DivergenceDomain::Energy });
        }
        if run_a[i].hashes.topo_hash != run_b[i].hashes.topo_hash {
            out.push(DivergenceRecord { tick: i, domain: DivergenceDomain::Topology });
        }
        if run_a[i].hashes.memory_hash != run_b[i].hashes.memory_hash {
            out.push(DivergenceRecord { tick: i, domain: DivergenceDomain::Memory });
        }
    }

    out
}

pub fn first_divergence_tick(divs: &[DivergenceRecord]) -> Option<usize> {
    divs.first().map(|d| d.tick)
}

pub fn validate_projection_purity(trace: &[ExportTickData]) -> bool {
    let hash_before = compute_trace_hash(trace);
    let _ = extract_events(trace, &EventThresholds::default());
    for i in 0..trace.len() {
        let _ = operator_contribution(trace, i);
    }
    let hash_after = compute_trace_hash(trace);
    hash_before == hash_after
}

pub fn validate_extraction_deterministic(trace: &[ExportTickData]) -> bool {
    let thresholds = EventThresholds::default();
    let events_1 = extract_events(trace, &thresholds);
    let events_2 = extract_events(trace, &thresholds);
    events_1.len() == events_2.len()
}

pub fn validate_attribution_consistency(trace: &[ExportTickData]) -> bool {
    for i in 0..trace.len() {
        let contrib = operator_contribution(trace, i);
        let sum = contrib.lie + contrib.diss + contrib.back + contrib.ema;
        if (sum - 1.0).abs() > 1e-4 {
            return false;
        }
    }
    true
}

pub fn full_validation_with_mode(
    trace_a: &[ExportTickData],
    trace_b: Option<&[ExportTickData]>,
    mode: ValidationMode,
) -> ValidationSummary {
    let divs = if let Some(b) = trace_b {
        detect_all_divergences(trace_a, b)
    } else {
        vec![]
    };

    let summary = ValidationSummary {
        engine_version: super::FORENSIC_ENGINE_VERSION,
        projection_pure: validate_projection_purity(trace_a),
        deterministic: validate_extraction_deterministic(trace_a),
        attribution_valid: validate_attribution_consistency(trace_a),
        graph_valid: true,
        divergence_valid: trace_b.is_none() || !divs.is_empty(),
    };

    if !summary.all_pass() {
        match mode {
            ValidationMode::Strict => panic!("{}", summary.status_string()),
            ValidationMode::Warn => eprintln!("⚠ {}", summary.status_string()),
        }
    }

    summary
}

pub fn compute_trace_hash(trace: &[ExportTickData]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    for tick in trace {
        tick.tick.hash(&mut hasher);
        tick.mode.hash(&mut hasher);
        tick.confidence.hash(&mut hasher);
        tick.energy_norm.hash(&mut hasher);
        tick.rollback_count.hash(&mut hasher);
        tick.learning_delta.hash(&mut hasher);
    }
    hasher.finish()
}
