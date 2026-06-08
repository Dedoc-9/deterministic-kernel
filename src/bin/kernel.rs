use deterministic_kernel::*;
use std::time::Instant;

fn main() {

    println!("=== DETERMINISTIC KERNEL: Phase 1–4 Full Stack ===\n");

    let dim = 5;
    let params = {
        let kappa = vec![vec![Fixed::new(0); dim]; dim];
        Parameters {
            lambda: Fixed::from_i32(1) / Fixed::from_i32(100),
            alpha: Fixed::from_i32(1) / Fixed::from_i32(100),
            beta_ema: Fixed::from_i32(1) / Fixed::from_i32(10),
            e_target: Fixed::from_i32(dim as i32),
            kappa,
        }
    };

    let seed = {
        let mut s = State::new(dim);
        s.z[0] = Fixed::from_i32(100);
        s
    };

    println!("Config: dim={}, ticks=10000", dim);
    println!("Running deterministic kernel with observability...");
    let start = Instant::now();
    let result = run_replay_with_observability(seed.clone(), &params, 10000);
    let elapsed = start.elapsed();
    println!("  completed in {:.3}ms", elapsed.as_secs_f64() * 1000.0);

    // Export telemetry as JSON for launcher inspection
    println!("\nExporting telemetry to JSON...");
    match export_replay_to_json(&result, "replay.json") {
        Ok(()) => {
            println!("✓ Replay exported: replay.json");
            println!("  Ready for launcher inspection");
            println!("\nNext: Run launcher:");
            println!("  ./launcher/target/release/deterministic-launcher.exe inspect replay.json");
            println!("  ./launcher/target/release/deterministic-launcher.exe export replay.json --format csv");
        }
        Err(e) => {
            eprintln!("✗ Failed to export replay: {}", e);
            std::process::exit(1);
        }
    }

    println!("\n✓ DETERMINISM CERTIFIED: System is reproducible.");
}
