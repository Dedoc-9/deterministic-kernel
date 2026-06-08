use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::BufReader;

// Export these for timeline_tui
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExportHashes {
    pub struct_hash: String,
    pub energy_hash: String,
    pub topo_hash: String,
    pub memory_hash: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExportTickData {
    pub tick: u64,
    pub mode: u8,
    pub confidence: u8,
    pub energy_norm: i64,
    pub rollback_count: u32,
    pub learning_delta: i64,
    pub hashes: ExportHashes,
}

// Intermediate struct for JSON deserialization (kernel exports hex strings)
#[derive(Serialize, Deserialize, Debug, Clone)]
struct JsonTickData {
    pub tick: u64,
    pub mode: u8,
    pub confidence: u8,
    pub energy_norm: i64,
    pub rollback_count: u32,
    pub learning_delta: i64,
    pub hashes: JsonHashes,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct JsonHashes {
    pub struct_hash: String,
    pub energy_hash: String,
    pub topo_hash: String,
    pub memory_hash: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct JsonReplayResult {
    pub ticks: Vec<JsonTickData>,
    pub trace_hash: String,
}

// Internal struct (uses byte arrays)
#[derive(Debug, Clone)]
pub struct TickData {
    pub tick: u64,
    pub mode: u8,
    pub confidence: u8,
    pub energy_norm: i64,
    pub rollback_count: u32,
    pub learning_delta: i64,
    pub hashes: Hashes,
}

#[derive(Debug, Clone)]
pub struct Hashes {
    pub struct_hash: [u8; 32],
    pub energy_hash: [u8; 32],
    pub topo_hash: [u8; 32],
    pub memory_hash: [u8; 32],
}

#[derive(Debug, Clone)]
pub struct ReplayResult {
    pub ticks: Vec<TickData>,
    pub trace_hash: String,  // Keep as hex string for comparison
}

/// Convert hex string to [u8; 32]
fn hex_to_bytes(hex: &str) -> Result<[u8; 32], Box<dyn std::error::Error>> {
    let mut bytes = [0u8; 32];
    for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
        if i >= 32 {
            break;
        }
        let hex_str = std::str::from_utf8(chunk)?;
        bytes[i] = u8::from_str_radix(hex_str, 16)?;
    }
    Ok(bytes)
}

pub fn load_replay(path: &str) -> Result<ReplayResult, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let json_replay: JsonReplayResult = serde_json::from_reader(reader)?;

    let ticks = json_replay
        .ticks
        .iter()
        .map(|t| TickData {
            tick: t.tick,
            mode: t.mode,
            confidence: t.confidence,
            energy_norm: t.energy_norm,
            rollback_count: t.rollback_count,
            learning_delta: t.learning_delta,
            hashes: Hashes {
                struct_hash: hex_to_bytes(&t.hashes.struct_hash).unwrap_or([0; 32]),
                energy_hash: hex_to_bytes(&t.hashes.energy_hash).unwrap_or([0; 32]),
                topo_hash: hex_to_bytes(&t.hashes.topo_hash).unwrap_or([0; 32]),
                memory_hash: hex_to_bytes(&t.hashes.memory_hash).unwrap_or([0; 32]),
            },
        })
        .collect();

    Ok(ReplayResult {
        ticks,
        trace_hash: json_replay.trace_hash,
    })
}

/// Convert mode number to readable string
pub fn mode_name(mode: u8) -> &'static str {
    match mode {
        0 => "Full",
        1 => "Damped",
        2 => "Frozen",
        3 => "Hold",
        _ => "Unknown",
    }
}

/// Convert hash to hex string for display
pub fn hash_to_hex(hash: &[u8; 32]) -> String {
    hash.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join("")
}

/// Load replay as export format (for timeline_tui and other tools)
pub fn load(path: &str) -> Vec<ExportTickData> {
    let data = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path, e));
    let json: serde_json::Value = serde_json::from_str(&data)
        .unwrap_or_else(|e| panic!("Failed to parse JSON: {}", e));

    let ticks = json["ticks"].as_array()
        .unwrap_or_else(|| panic!("No 'ticks' array in JSON"));

    ticks.iter().map(|t| {
        ExportTickData {
            tick: t["tick"].as_u64().unwrap_or(0),
            mode: t["mode"].as_u64().unwrap_or(0) as u8,
            confidence: t["confidence"].as_u64().unwrap_or(0) as u8,
            energy_norm: t["energy_norm"].as_i64().unwrap_or(0),
            rollback_count: t["rollback_count"].as_u64().unwrap_or(0) as u32,
            learning_delta: t["learning_delta"].as_i64().unwrap_or(0),
            hashes: ExportHashes {
                struct_hash: t["hashes"]["struct_hash"].as_str().unwrap_or("0").to_string(),
                energy_hash: t["hashes"]["energy_hash"].as_str().unwrap_or("0").to_string(),
                topo_hash: t["hashes"]["topo_hash"].as_str().unwrap_or("0").to_string(),
                memory_hash: t["hashes"]["memory_hash"].as_str().unwrap_or("0").to_string(),
            },
        }
    }).collect()
}
