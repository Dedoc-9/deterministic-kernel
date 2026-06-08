use crate::telemetry_reader::{ReplayResult, hash_to_hex};

pub fn show_integrity(replay: &ReplayResult) {
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║              INTEGRITY INSPECTOR (4-DOMAIN HASHING)             ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    println!("Hash Sample (first 5 ticks):\n");
    println!("Tick │ Struct Hash (first 16 chars) │ Energy Hash │ Topo Hash │ Memory Hash");
    println!("─────┼─────────────────────────────┼─────────────┼───────────┼────────────");

    for t in replay.ticks.iter().take(5) {
        let struct_short = hash_to_hex(&t.hashes.struct_hash)[..16].to_string();
        let energy_short = hash_to_hex(&t.hashes.energy_hash)[..16].to_string();
        let topo_short = hash_to_hex(&t.hashes.topo_hash)[..16].to_string();
        let memory_short = hash_to_hex(&t.hashes.memory_hash)[..16].to_string();

        println!(
            "{:4} │ {} │ {} │ {} │ {}",
            t.tick, struct_short, energy_short, topo_short, memory_short
        );
    }

    if replay.ticks.len() > 5 {
        println!("     ... ({} more ticks)", replay.ticks.len() - 5);
    }

    println!("\n");
    println!("Confidence Weighting (Phase 2a):");
    println!("  Structural (Z, S):      35%");
    println!("  Energy (||Z||², ||S||²): 25%");
    println!("  Topological (operator):  25%");
    println!("  Memory (Z ⊕ S):          15%");
    println!("  ───────────────────────────");
    println!("  Total:                  100%");

    println!("\nConfidence History (sample):");
    for t in replay.ticks.iter().take(10) {
        let bar_len = (t.confidence / 10) as usize;
        let bar = "█".repeat(bar_len) + &"░".repeat(10 - bar_len);
        println!("  Tick {:4}: {} {:>3}%", t.tick, bar, t.confidence);
    }

    if replay.ticks.len() > 10 {
        println!("  ... ({} more ticks)", replay.ticks.len() - 10);
    }

    println!("\n");
}
