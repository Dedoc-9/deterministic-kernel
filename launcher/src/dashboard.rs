use crate::telemetry_reader::{ReplayResult, mode_name};

pub fn show_dashboard(replay: &ReplayResult) {
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║           DETERMINISTIC KERNEL TELEMETRY DASHBOARD             ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    println!("Tick │ Mode    │ Conf │ Energy Norm   │ Rollback │ Learning Δ");
    println!("─────┼─────────┼──────┼───────────────┼──────────┼────────────");

    for t in &replay.ticks {
        let mode_str = mode_name(t.mode);
        println!(
            "{:4} │ {:<7} │ {:>3}% │ {:>13} │ {:>8} │ {:>10}",
            t.tick, mode_str, t.confidence, t.energy_norm, t.rollback_count, t.learning_delta
        );
    }

    println!("\n");
    println!("Summary:");
    println!("  Total ticks: {}", replay.ticks.len());

    // Mode statistics
    let mode_counts = replay.ticks.iter().fold([0; 4], |mut acc, t| {
        acc[t.mode as usize] += 1;
        acc
    });

    println!("  Mode distribution:");
    println!("    Full:   {} ticks", mode_counts[0]);
    println!("    Damped: {} ticks", mode_counts[1]);
    println!("    Frozen: {} ticks", mode_counts[2]);
    println!("    Hold:   {} ticks", mode_counts[3]);

    // Confidence statistics
    let avg_confidence = if !replay.ticks.is_empty() {
        replay.ticks.iter().map(|t| t.confidence as u32).sum::<u32>() / replay.ticks.len() as u32
    } else {
        0
    };
    let min_confidence = replay.ticks.iter().map(|t| t.confidence).min().unwrap_or(0);
    let max_confidence = replay.ticks.iter().map(|t| t.confidence).max().unwrap_or(0);

    println!("  Confidence: avg={}, min={}, max={}", avg_confidence, min_confidence, max_confidence);

    // Energy statistics
    let max_energy = replay.ticks.iter().map(|t| t.energy_norm).max().unwrap_or(0);
    let min_energy = replay.ticks.iter().map(|t| t.energy_norm).min().unwrap_or(0);

    println!("  Energy norm: min={}, max={}", min_energy, max_energy);

    // Rollback events
    let rollback_count = replay.ticks.iter().filter(|t| t.rollback_count > 0).count();
    println!("  Rollback events: {}", rollback_count);

    println!("\n");
}
