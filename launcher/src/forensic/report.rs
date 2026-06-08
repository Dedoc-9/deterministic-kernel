//! Forensic report generation and export

use crate::forensic::validation::{full_validation_with_mode, ValidationMode, ValidationSummary};
use crate::forensics::{ExportReplayResult, extract_events, build_temporal_graph, EventThresholds};
use std::fs::File;
use std::io::Write;

#[derive(Debug)]
pub struct ForensicReport {
    pub engine_version: String,
    pub trace_hash: u64,
    pub validation: ValidationSummary,
    pub event_count: usize,
    pub divergence_tick: Option<usize>,
}

pub fn build_report(trace: &ExportReplayResult) -> ForensicReport {
    let validation = full_validation_with_mode(&trace.ticks, None, ValidationMode::Warn);
    let events = extract_events(&trace.ticks, &EventThresholds::default());
    let _graph = build_temporal_graph(&events, 50);
    let trace_hash = crate::forensic::validation::compute_trace_hash(&trace.ticks);

    ForensicReport {
        engine_version: crate::forensic::FORENSIC_ENGINE_VERSION.to_string(),
        trace_hash,
        validation,
        event_count: events.len(),
        divergence_tick: None,
    }
}

/// Pure projection: serialize report to JSON (no recomputation)
pub fn to_json(report: &ForensicReport) -> String {
    format!(
        r#"{{
  "engine_version": "{}",
  "trace_hash": {},
  "event_count": {},
  "validation": {{
    "projection_pure": {},
    "deterministic": {},
    "attribution_valid": {},
    "graph_valid": {},
    "divergence_valid": {}
  }},
  "status_string": "{}"
}}"#,
        report.engine_version,
        report.trace_hash,
        report.event_count,
        report.validation.projection_pure,
        report.validation.deterministic,
        report.validation.attribution_valid,
        report.validation.graph_valid,
        report.validation.divergence_valid,
        report.validation.status_string()
    )
}

/// Pure projection: export report to JSON file
pub fn write_json(path: &str, report: &ForensicReport) {
    let mut file = File::create(path).expect("failed to create json file");
    let json = to_json(report);
    file.write_all(json.as_bytes())
        .expect("failed to write json");
}

/// Pure projection: export report to CSV file
pub fn write_csv(path: &str, report: &ForensicReport) {
    let mut file = File::create(path).expect("failed to create csv file");

    let header = "engine_version,trace_hash,event_count,projection,deterministic,attribution,graph,divergence\n";
    file.write_all(header.as_bytes()).expect("failed to write header");

    let line = format!(
        "{},{},{},{},{},{},{},{}\n",
        report.engine_version,
        report.trace_hash,
        report.event_count,
        report.validation.projection_pure,
        report.validation.deterministic,
        report.validation.attribution_valid,
        report.validation.graph_valid,
        report.validation.divergence_valid,
    );

    file.write_all(line.as_bytes()).expect("failed to write csv");
}
