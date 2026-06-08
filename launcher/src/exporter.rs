use crate::telemetry_reader::{ReplayResult, hash_to_hex, mode_name};
use std::fs::File;
use std::io::Write;

pub fn export(replay: &ReplayResult, format: &str) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        "csv" => export_csv(replay)?,
        "json" => export_json(replay)?,
        _ => {
            eprintln!("Unsupported format: {}. Use 'csv' or 'json'", format);
            return Err("Unsupported format".into());
        }
    }

    println!("✓ Exported replay as {}", format);
    Ok(())
}

fn export_csv(replay: &ReplayResult) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create("audit_trail.csv")?;

    // Write header
    writeln!(
        file,
        "tick,mode,confidence,energy_norm,rollback_count,learning_delta,struct_hash,energy_hash,topo_hash,memory_hash"
    )?;

    // Write data rows
    for t in &replay.ticks {
        let mode_str = mode_name(t.mode);
        writeln!(
            file,
            "{},{},{},{},{},{},{},{},{},{}",
            t.tick,
            mode_str,
            t.confidence,
            t.energy_norm,
            t.rollback_count,
            t.learning_delta,
            hash_to_hex(&t.hashes.struct_hash),
            hash_to_hex(&t.hashes.energy_hash),
            hash_to_hex(&t.hashes.topo_hash),
            hash_to_hex(&t.hashes.memory_hash)
        )?;
    }

    println!("  Output: audit_trail.csv ({} ticks)", replay.ticks.len());
    Ok(())
}

fn export_json(replay: &ReplayResult) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create("audit_trail.json")?;

    // Convert hashes to hex for JSON export
    #[derive(serde::Serialize)]
    struct ExportTick {
        tick: u64,
        mode: String,
        confidence: u8,
        energy_norm: i64,
        rollback_count: u32,
        learning_delta: i64,
        struct_hash: String,
        energy_hash: String,
        topo_hash: String,
        memory_hash: String,
    }

    let export_ticks: Vec<ExportTick> = replay
        .ticks
        .iter()
        .map(|t| ExportTick {
            tick: t.tick,
            mode: mode_name(t.mode).to_string(),
            confidence: t.confidence,
            energy_norm: t.energy_norm,
            rollback_count: t.rollback_count,
            learning_delta: t.learning_delta,
            struct_hash: hash_to_hex(&t.hashes.struct_hash),
            energy_hash: hash_to_hex(&t.hashes.energy_hash),
            topo_hash: hash_to_hex(&t.hashes.topo_hash),
            memory_hash: hash_to_hex(&t.hashes.memory_hash),
        })
        .collect();

    serde_json::to_writer_pretty(file, &export_ticks)?;

    println!("  Output: audit_trail.json ({} ticks)", replay.ticks.len());
    Ok(())
}
