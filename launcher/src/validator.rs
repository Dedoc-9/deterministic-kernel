use crate::telemetry_reader::load_replay;

pub fn verify_traces(path1: &str, path2: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║            DETERMINISM VERIFICATION (Trace Comparison)           ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    let replay1 = load_replay(path1)?;
    let replay2 = load_replay(path2)?;

    println!("Run 1: {}", path1);
    println!("  Ticks: {}", replay1.ticks.len());
    println!("  Trace Hash: {}", replay1.trace_hash);

    println!("\nRun 2: {}", path2);
    println!("  Ticks: {}", replay2.ticks.len());
    println!("  Trace Hash: {}", replay2.trace_hash);

    println!("\n");

    // Compare trace hashes
    if replay1.trace_hash == replay2.trace_hash {
        println!("✓ PASS: Trace hashes match");
        println!("  Determinism verified: identical execution across runs");
        println!("\n  Status: DETERMINISM CERTIFIED");
    } else {
        println!("✗ FAIL: Trace hashes DO NOT match");
        println!("  System is non-deterministic or inputs differ");

        // Find first divergence
        if replay1.ticks.len() != replay2.ticks.len() {
            println!(
                "  Tick count mismatch: {} vs {}",
                replay1.ticks.len(),
                replay2.ticks.len()
            );
        } else {
            // Compare tick by tick
            for (i, (t1, t2)) in replay1.ticks.iter().zip(replay2.ticks.iter()).enumerate() {
                if t1.energy_norm != t2.energy_norm || t1.mode != t2.mode {
                    println!("  First divergence at tick {}", i);
                    println!(
                        "    Run 1: mode={}, energy={}",
                        t1.mode, t1.energy_norm
                    );
                    println!(
                        "    Run 2: mode={}, energy={}",
                        t2.mode, t2.energy_norm
                    );
                    break;
                }
            }
        }

        println!("\n  Status: NON-DETERMINISTIC (audit required)");
    }

    println!("\n");
    Ok(())
}
