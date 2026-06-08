mod telemetry_reader;
mod dashboard;
mod integrity;
mod exporter;
mod validator;
mod timeline_tui;
mod forensic;
mod forensics;
mod cognitive;
mod observatory;

use std::env;
use observatory::ObservatoryApp;
use cognitive::ObservatoryViewModel;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("inspect") => {
            if let Some(file) = args.get(2) {
                match telemetry_reader::load_replay(file) {
                    Ok(replay) => {
                        dashboard::show_dashboard(&replay);
                        integrity::show_integrity(&replay);
                    }
                    Err(e) => eprintln!("Error loading replay: {}", e),
                }
            } else {
                eprintln!("Usage: deterministic-launcher inspect <replay_file>");
            }
        }
        Some("export") => {
            if let Some(file) = args.get(2) {
                let format = args.get(4).map(|s| s.as_str()).unwrap_or("csv");
                match telemetry_reader::load_replay(file) {
                    Ok(replay) => {
                        if let Err(e) = exporter::export(&replay, format) {
                            eprintln!("Error exporting: {}", e);
                        }
                    }
                    Err(e) => eprintln!("Error loading replay: {}", e),
                }
            } else {
                eprintln!("Usage: deterministic-launcher export <replay_file> --format <csv|json>");
            }
        }
        Some("verify") => {
            if let (Some(file1), Some(file2)) = (args.get(2), args.get(3)) {
                if let Err(e) = validator::verify_traces(file1, file2) {
                    eprintln!("Error verifying: {}", e);
                }
            } else {
                eprintln!("Usage: deterministic-launcher verify <run1> <run2>");
            }
        }
        Some("timeline") => {
            if let (Some(file1), Some(file2)) = (args.get(2), args.get(3)) {
                timeline_tui::run(file1, file2);
            } else {
                eprintln!("Usage: deterministic-launcher timeline <run1> <run2>");
            }
        }
        Some("validate") => {
            if let Some(path) = args.get(2) {
                let ticks = telemetry_reader::load(path);
                let trace = forensics::ExportReplayResult {
                    ticks,
                    trace_hash: "forensic-validation".to_string(),
                };
                let report = forensic::report::build_report(&trace);
                println!("FORENSIC REPORT");
                println!("Engine: {}", report.engine_version);
                println!("Trace hash: {}", report.trace_hash);
                println!("Events: {}", report.event_count);
                println!("{}", report.validation.status_string());
            } else {
                eprintln!("Usage: deterministic-launcher validate <trace.json>");
            }
        }
        Some("observatory") => {
            if let Some(trace_path) = args.get(2) {
                let ticks = telemetry_reader::load(trace_path);
                let run_a = forensics::ExportReplayResult {
                    ticks,
                    trace_hash: "observatory-run".to_string(),
                };
                let report = forensic::report::build_report(&run_a);
                if !report.validation.all_pass() {
                    eprintln!("⚠ FORENSIC VALIDATION FAILED");
                    eprintln!("{}", report.validation.status_string());
                    panic!("Observatory halted: invalid forensic state");
                }
                println!("Observatory loaded: trace OK");
                println!("  Engine: {}", report.engine_version);
                println!("  Events: {}", report.event_count);
                println!("  Status: ✓ TRUSTED");
            } else {
                eprintln!("Usage: deterministic-launcher observatory <trace.json>");
            }
        }
        Some("forensic-export") => {
            if let Some(path) = args.get(2) {
                let ticks = telemetry_reader::load(path);
                let trace = forensics::ExportReplayResult {
                    ticks,
                    trace_hash: "forensic-export".to_string(),
                };
                let report = forensic::report::build_report(&trace);

                let json_path = format!("{}.forensic.json", path);
                let csv_path = format!("{}.forensic.csv", path);

                forensic::report::write_json(&json_path, &report);
                forensic::report::write_csv(&csv_path, &report);

                println!("Forensic export complete:");
                println!("  JSON → {}", json_path);
                println!("  CSV  → {}", csv_path);
            } else {
                eprintln!("Usage: deterministic-launcher forensic-export <trace.json>");
            }
        }
        Some("consensus") => {
            let trace_paths: Vec<String> = args.iter().skip(2).cloned().collect();
            if trace_paths.is_empty() {
                eprintln!("Usage: deterministic-launcher consensus <run1.json> <run2.json> [run3.json ...]");
                return;
            }

            let mut runs_data = Vec::new();
            for path in &trace_paths {
                let ticks = telemetry_reader::load(path);
                runs_data.push(ticks);
            }

            let analysis = forensic::consensus::ConsensusAnalysis::new(runs_data);

            println!("Consensus Analysis:");
            println!("  Runs loaded: {}", analysis.runs.len());
            println!("  Divergence clusters: {}", analysis.divergence_count());
            println!("  Operator consensus ticks: {}", analysis.consensus_tick_count());

            if analysis.cluster_metrics.len() > 0 {
                println!("\nCluster Metrics:");
                for metric in analysis.cluster_metrics.iter().take(5) {
                    println!("  [{}] tick {}-{}: agreement={:.4}, domains={:?}",
                        metric.cluster_id, metric.start_tick, metric.end_tick, metric.avg_agreement, metric.domain_set);
                }
            }
        }
        Some("consensus-export") => {
            let trace_paths: Vec<String> = args.iter().skip(2).cloned().collect();
            if trace_paths.is_empty() {
                eprintln!("Usage: deterministic-launcher consensus-export <run1.json> <run2.json> [run3.json ...]");
                return;
            }

            let mut runs_data = Vec::new();
            for path in &trace_paths {
                let ticks = telemetry_reader::load(path);
                runs_data.push(ticks);
            }

            let analysis = forensic::consensus::ConsensusAnalysis::new(runs_data);

            let json_path = format!("{}.consensus.json", trace_paths[0]);
            let csv_path = format!("{}.consensus.csv", trace_paths[0]);

            forensic::consensus::write_consensus_json(&json_path, &analysis);
            forensic::consensus::write_consensus_csv(&csv_path, &analysis);

            println!("Consensus export complete:");
            println!("  JSON → {}", json_path);
            println!("  CSV  → {}", csv_path);
            println!("  Clusters merged: {}", analysis.divergence_clusters.len());
            println!("  Avg agreement: {:.4}",
                if analysis.cluster_metrics.is_empty() { 1.0 }
                else { analysis.cluster_metrics.iter().map(|m| m.avg_agreement).sum::<f32>() / analysis.cluster_metrics.len() as f32 }
            );
        }
        Some("observatory-gui") => {
            let consensus_path = args.get(2).cloned();

            let app = if let Some(path) = consensus_path {
                match std::fs::read_to_string(&path) {
                    Ok(_data) => {
                        // Load from trace files — build frozen ConsensusAnalysis
                        let ticks = telemetry_reader::load(&path);
                        let analysis = forensic::consensus::ConsensusAnalysis::new(vec![ticks.clone(), ticks]);

                        // Wrap in ViewModel (separates forensic core from UI state)
                        let vm = ObservatoryViewModel::new(analysis);

                        ObservatoryApp { vm: Some(vm) }
                    }
                    Err(_) => ObservatoryApp::default(),
                }
            } else {
                ObservatoryApp::default()
            };

            let mut native_options = eframe::NativeOptions::default();
            native_options.viewport = egui::ViewportBuilder::default()
                .with_inner_size([1000.0, 700.0]);

            let _ = eframe::run_native(
                "Deterministic Forensic Observatory (Phase 5c)",
                native_options,
                Box::new(|_cc| Box::new(app)),
            );
        }
        _ => {
            eprintln!("Deterministic Kernel Launcher");
            eprintln!("\nCommands:");
            eprintln!("  inspect <replay_file>              - Display telemetry dashboard");
            eprintln!("  export <replay_file> --format csv  - Export to CSV");
            eprintln!("  export <replay_file> --format json - Export to JSON");
            eprintln!("  verify <run1> <run2>               - Compare trace hashes for determinism");
            eprintln!("  timeline <run1> <run2>             - Interactive observatory (operator heatmap, causal backtrace)");
            eprintln!("  validate <trace.json>              - Validate forensic trace");
            eprintln!("  observatory <trace.json>           - Load and validate observatory");
            eprintln!("  forensic-export <trace.json>       - Export forensic report (JSON/CSV)");
            eprintln!("  consensus <run1.json> <run2.json>  - Multi-run consensus analysis");
            eprintln!("  consensus-export <run1.json> ...   - Export consensus to JSON/CSV");
            eprintln!("  observatory-gui [trace.json]       - Launch windowed GUI with heatmaps");
        }
    }
}
