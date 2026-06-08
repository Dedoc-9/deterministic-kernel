//! Windowed forensic observatory with heatmap rendering and divergence cluster visualization
//! Phase 5c: Pure renderer using immutable forensic core via ViewModel

use eframe::egui;
use crate::cognitive::ObservatoryViewModel;

pub struct ObservatoryApp {
    pub vm: Option<ObservatoryViewModel>,
}

impl Default for ObservatoryApp {
    fn default() -> Self {
        Self { vm: None }
    }
}

impl eframe::App for ObservatoryApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🔬 Deterministic Forensic Observatory (Phase 5c)");

            if let Some(vm) = &mut self.vm {
                // Determinism badge
                if vm.divergence_cluster_count() == 0 {
                    ui.colored_label(egui::Color32::GREEN, "✔ DETERMINISTIC");
                } else {
                    ui.colored_label(egui::Color32::LIGHT_RED, "✖ DIVERGENCE DETECTED");
                }

                ui.horizontal(|ui| {
                    ui.label(format!("📊 Runs: {}", vm.run_count()));
                    ui.label(format!("🔴 Divergence clusters: {}", vm.divergence_cluster_count()));
                    ui.label(format!("📈 Consensus ticks: {}", vm.total_consensus_ticks()));
                });

                ui.separator();

                // Global playback slider (unified tick scrub)
                ui.horizontal(|ui| {
                    ui.label("Global Tick:");
                    let mut tick = vm.current_tick();
                    let max = vm.state.global.max;

                    if ui.add(egui::Slider::new(&mut tick, 0..=max)).changed() {
                        vm.set_tick(tick);
                    }
                    ui.label(format!("({}/{})", tick, max));
                });

                ui.separator();

                if vm.divergence_cluster_count() == 0 {
                    ui.label("✓ No divergences detected (runs are deterministic)");
                } else {
                    ui.label("Divergence Cluster Viewport:");

                    // Pre-collect cluster data to avoid borrow conflicts
                    let cluster_data: Vec<_> = vm.active_clusters()
                        .iter()
                        .enumerate()
                        .map(|(idx, cluster)| (idx, cluster.clone()))
                        .collect();

                    let mut selected_cluster = None;

                    egui::ScrollArea::vertical()
                        .max_height(300.0)
                        .show(ui, |ui| {
                            for (idx, cluster) in cluster_data.iter() {
                                ui.group(|ui| {
                                    let header_text = format!(
                                        "Cluster {}: ticks {}-{} ({} ticks) | Domains: {:?}",
                                        idx,
                                        cluster.start_tick,
                                        cluster.end_tick,
                                        cluster.end_tick - cluster.start_tick + 1,
                                        cluster.domains
                                    );

                                    let response = ui.selectable_label(
                                        vm.state.selected_cluster == Some(*idx),
                                        &header_text,
                                    );

                                    if response.clicked() {
                                        selected_cluster = Some(*idx);
                                    }

                                    // Operator heatmap bars
                                    if cluster.operator_intensity.len() >= 4 {
                                        ui.label("Operator Intensity:");
                                        let operators = ["LIE", "DISS", "BACK", "EMA"];
                                        for (&intensity, op_name) in cluster
                                            .operator_intensity
                                            .iter()
                                            .zip(operators.iter())
                                        {
                                            let (_id, rect) = ui.allocate_space(egui::vec2(150.0, 18.0));
                                            let color = Self::intensity_to_color(intensity);
                                            ui.painter().rect_filled(rect, 2.0, color);
                                            ui.label(format!("{}: {:.3}", op_name, intensity));
                                        }
                                    }

                                    ui.label("🔗 Causal backtrace (Phase 5d)");
                                });
                            }
                        });

                    // Apply cluster selection outside of borrow scope
                    if let Some(idx) = selected_cluster {
                        vm.set_selected_cluster(Some(idx));
                    }
                }

                // Phase 5d: Rollback intensity heatmap
                ui.separator();
                ui.label("🔥 Rollback Intensity Heatmap:");
                ui.horizontal(|ui| {
                    let sample_size = 100.min(vm.rollback_intensity.len());
                    for i in 0..sample_size {
                        let intensity = vm.rollback_intensity[i * vm.rollback_intensity.len() / sample_size];
                        let color = egui::Color32::from_rgb(
                            (intensity * 255.0) as u8,
                            (50.0 * (1.0 - intensity)) as u8,
                            ((1.0 - intensity) * 255.0) as u8,
                        );
                        ui.painter().rect_filled(
                            egui::Rect::from_min_size(
                                ui.cursor().left_top() + egui::vec2(i as f32 * 3.0, 0.0),
                                egui::vec2(2.5, 20.0),
                            ),
                            1.0,
                            color,
                        );
                    }
                    ui.label("(Red=high rollback, Blue=low)");
                });

                // Phase 5d: Causal backtrace visualization
                ui.separator();
                ui.label("🔗 Causal Backtrace Graph (Phase 5d):");
                let current_tick = vm.current_tick();
                let relevant_edges: Vec<_> = vm.backtrace_edges.iter()
                    .filter(|e| e.target_tick >= current_tick.saturating_sub(50) && e.target_tick <= current_tick + 50)
                    .take(10)
                    .collect();

                if relevant_edges.is_empty() {
                    ui.label("No causal edges in current tick window");
                } else {
                    egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                        for edge in relevant_edges {
                            ui.label(format!(
                                "🔀 {} → {} [strength: {:.2}]",
                                edge.source_tick, edge.target_tick, edge.strength
                            ));
                        }
                    });
                }

                // Phase 5d: Detailed cluster view
                if let Some(cluster_idx) = vm.state.selected_cluster {
                    if let Some(cluster) = vm.active_clusters().get(cluster_idx) {
                        ui.separator();
                        ui.heading(format!("📍 Cluster {} Details", cluster_idx));

                        ui.horizontal(|ui| {
                            ui.label(format!("Start: {}", cluster.start_tick));
                            ui.label(format!("End: {}", cluster.end_tick));
                            ui.label(format!("Duration: {} ticks", cluster.end_tick - cluster.start_tick + 1));
                        });

                        ui.label(format!("Domains: {:?}", cluster.domains));

                        // Phase 5d: Multi-run delta at this cluster
                        if !vm.multi_run_delta.is_empty() && cluster.start_tick < vm.multi_run_delta.len() {
                            let delta = vm.multi_run_delta[cluster.start_tick];
                            ui.colored_label(
                                if delta > 0.5 {
                                    egui::Color32::YELLOW
                                } else {
                                    egui::Color32::GREEN
                                },
                                format!("Cross-run variance: {:.3}", delta),
                            );
                        }
                    }
                }

                // Phase 5d: Snapshot export
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("📸 Export SVG Snapshot").clicked() {
                        Self::export_svg(vm, current_tick);
                    }
                    if ui.button("💾 Export JSON Report").clicked() {
                        Self::export_json_report(vm, current_tick);
                    }
                });

            } else {
                ui.label("ℹ No ViewModel loaded.");
                ui.label("Usage: cargo run -- observatory-gui <trace.json>");
            }
        });
    }
}

impl ObservatoryApp {
    /// Convert operator intensity [0,1] to color gradient (blue → red)
    fn intensity_to_color(intensity: f32) -> egui::Color32 {
        let clamped = intensity.max(0.0).min(1.0);
        let r = (clamped * 255.0) as u8;
        let b = (255.0 * (1.0 - clamped)) as u8;
        egui::Color32::from_rgb(r, 0, b)
    }

    /// Phase 5d: Export forensic snapshot as SVG (deterministic, read-only)
    fn export_svg(vm: &crate::cognitive::ObservatoryViewModel, tick: usize) {
        use std::fs::File;
        use std::io::Write;

        let filename = format!("observatory_tick_{}.svg", tick);
        let determinism = if vm.divergence_cluster_count() == 0 {
            "DETERMINISTIC"
        } else {
            "DIVERGENCE"
        };

        let line1 = format!("Engine: 2V.1 | Tick: {} | Runs: {} | Clusters: {}",
            tick, vm.run_count(), vm.divergence_cluster_count());
        let line2 = format!("Determinism: {}", determinism);

        let mut svg = String::new();
        svg.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        svg.push_str("<svg width=\"800\" height=\"600\" xmlns=\"http://www.w3.org/2000/svg\">\n");
        svg.push_str("  <rect width=\"800\" height=\"600\" fill=\"white\"/>\n");
        svg.push_str("  <text x=\"10\" y=\"30\">Deterministic Forensic Observatory Phase 5d</text>\n");
        svg.push_str(&format!("  <text x=\"10\" y=\"60\">{}</text>\n", line1));
        svg.push_str(&format!("  <text x=\"10\" y=\"90\">{}</text>\n", line2));
        svg.push_str("  <rect x=\"10\" y=\"110\" width=\"780\" height=\"400\" fill=\"#f0f0f0\"/>\n");
        svg.push_str("  <text x=\"20\" y=\"130\">Causal Backtrace Graph</text>\n");
        svg.push_str("  <text x=\"20\" y=\"500\">Multi-run Divergence Analysis</text>\n");
        svg.push_str("</svg>\n");

        if let Ok(mut file) = File::create(&filename) {
            let _ = file.write_all(svg.as_bytes());
            eprintln!("SVG exported: {}", filename);
        } else {
            eprintln!("Failed to write SVG: {}", filename);
        }
    }

    /// Phase 5d: Export forensic report as JSON (deterministic, read-only)
    fn export_json_report(vm: &crate::cognitive::ObservatoryViewModel, tick: usize) {
        use std::fs::File;
        use std::io::Write;

        let filename = format!("observatory_report_tick_{:06}.json", tick);
        let report = format!(
            r#"{{
  "engine": "2V.1",
  "phase": "5d",
  "tick": {},
  "runs": {},
  "divergence_clusters": {},
  "determinism": {},
  "backtrace_edges": {},
  "rollback_max": {:.3},
  "multi_run_delta_max": {:.3}
}}"#,
            tick,
            vm.run_count(),
            vm.divergence_cluster_count(),
            if vm.divergence_cluster_count() == 0 { "true" } else { "false" },
            vm.backtrace_edges.len(),
            vm.rollback_intensity.iter().cloned().fold(0.0f32, f32::max),
            vm.multi_run_delta.iter().cloned().fold(0.0f32, f32::max),
        );

        if let Ok(mut file) = File::create(&filename) {
            let _ = file.write_all(report.as_bytes());
            eprintln!("JSON report exported: {}", filename);
        } else {
            eprintln!("Failed to write JSON: {}", filename);
        }
    }
}
