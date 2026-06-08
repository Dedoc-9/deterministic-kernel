//! Synthetic trace fixtures for forensic validation

use crate::telemetry_reader::{ExportHashes, ExportTickData};
use crate::forensics::ExportReplayResult;

fn make_hash(region: &str) -> ExportHashes {
    let base = region.as_bytes()[0] as u32;
    ExportHashes {
        struct_hash: format!("{:032x}", base.wrapping_mul(1)),
        energy_hash: format!("{:032x}", base.wrapping_mul(2)),
        topo_hash: format!("{:032x}", base.wrapping_mul(3)),
        memory_hash: format!("{:032x}", base.wrapping_mul(4)),
    }
}

pub fn stable_equilibrium() -> ExportReplayResult {
    let ticks: Vec<ExportTickData> = (0..1000)
        .map(|i| ExportTickData {
            tick: i as u64,
            mode: 0,
            confidence: 100,
            energy_norm: 1_000_000,
            rollback_count: 0,
            learning_delta: 0,
            hashes: make_hash("stable"),
        })
        .collect();

    ExportReplayResult {
        ticks,
        trace_hash: "stable_equilibrium".to_string(),
    }
}

pub fn confidence_collapse() -> ExportReplayResult {
    let mut ticks = Vec::new();

    for i in 0..100 {
        ticks.push(ExportTickData {
            tick: i as u64,
            mode: 0,
            confidence: 100,
            energy_norm: 1_000_000,
            rollback_count: 0,
            learning_delta: 0,
            hashes: make_hash("stable"),
        });
    }

    for i in 100..150 {
        let conf = (100 - ((i - 100) * 2)) as u8;
        ticks.push(ExportTickData {
            tick: i as u64,
            mode: 0,
            confidence: conf,
            energy_norm: 1_000_000,
            rollback_count: 0,
            learning_delta: 0,
            hashes: make_hash("collapse"),
        });
    }

    for i in 150..250 {
        ticks.push(ExportTickData {
            tick: i as u64,
            mode: 0,
            confidence: 50,
            energy_norm: 1_000_000,
            rollback_count: 0,
            learning_delta: 0,
            hashes: make_hash("recovery"),
        });
    }

    ExportReplayResult {
        ticks,
        trace_hash: "confidence_collapse".to_string(),
    }
}

pub fn energy_spike() -> ExportReplayResult {
    let mut ticks = Vec::new();

    for i in 0..100 {
        ticks.push(ExportTickData {
            tick: i as u64,
            mode: 0,
            confidence: 100,
            energy_norm: 1_000_000,
            rollback_count: 0,
            learning_delta: 0,
            hashes: make_hash("stable"),
        });
    }

    for i in 100..120 {
        let energy = 1_000_000 + ((i - 100) as i64 * 5_000_000);
        ticks.push(ExportTickData {
            tick: i as u64,
            mode: 0,
            confidence: 100,
            energy_norm: energy,
            rollback_count: 0,
            learning_delta: 0,
            hashes: make_hash("spike"),
        });
    }

    for i in 120..200 {
        ticks.push(ExportTickData {
            tick: i as u64,
            mode: 0,
            confidence: 100,
            energy_norm: 2_000_000,
            rollback_count: 0,
            learning_delta: 0,
            hashes: make_hash("settled"),
        });
    }

    ExportReplayResult {
        ticks,
        trace_hash: "energy_spike".to_string(),
    }
}

pub fn rollback_cascade() -> ExportReplayResult {
    let mut ticks = Vec::new();

    for i in 0..100 {
        ticks.push(ExportTickData {
            tick: i as u64,
            mode: 0,
            confidence: 100,
            energy_norm: 1_000_000,
            rollback_count: 0,
            learning_delta: 0,
            hashes: make_hash("stable"),
        });
    }

    for i in 100..120 {
        ticks.push(ExportTickData {
            tick: i as u64,
            mode: 0,
            confidence: 100,
            energy_norm: 1_000_000,
            rollback_count: (i - 100) as u32,
            learning_delta: 0,
            hashes: make_hash("cascade"),
        });
    }

    for i in 120..200 {
        ticks.push(ExportTickData {
            tick: i as u64,
            mode: 0,
            confidence: 100,
            energy_norm: 1_000_000,
            rollback_count: 10,
            learning_delta: 0,
            hashes: make_hash("recovery"),
        });
    }

    ExportReplayResult {
        ticks,
        trace_hash: "rollback_cascade".to_string(),
    }
}

pub fn mode_oscillation() -> ExportReplayResult {
    let ticks: Vec<ExportTickData> = (0..200)
        .map(|i| ExportTickData {
            tick: i as u64,
            mode: (i % 2) as u8,
            confidence: 100,
            energy_norm: 1_000_000,
            rollback_count: 0,
            learning_delta: 0,
            hashes: make_hash("oscillation"),
        })
        .collect();

    ExportReplayResult {
        ticks,
        trace_hash: "mode_oscillation".to_string(),
    }
}

pub fn learning_burst() -> ExportReplayResult {
    let mut ticks = Vec::new();

    for i in 0..100 {
        ticks.push(ExportTickData {
            tick: i as u64,
            mode: 0,
            confidence: 100,
            energy_norm: 1_000_000,
            rollback_count: 0,
            learning_delta: 0,
            hashes: make_hash("stable"),
        });
    }

    for i in 100..120 {
        let delta = (i - 100) as i64 * 10_000;
        ticks.push(ExportTickData {
            tick: i as u64,
            mode: 0,
            confidence: 100,
            energy_norm: 1_000_000,
            rollback_count: 0,
            learning_delta: delta,
            hashes: make_hash("burst"),
        });
    }

    for i in 120..200 {
        ticks.push(ExportTickData {
            tick: i as u64,
            mode: 0,
            confidence: 100,
            energy_norm: 1_000_000,
            rollback_count: 0,
            learning_delta: 50000,
            hashes: make_hash("settled"),
        });
    }

    ExportReplayResult {
        ticks,
        trace_hash: "learning_burst".to_string(),
    }
}

pub fn noise_floor_trace() -> ExportReplayResult {
    let ticks: Vec<ExportTickData> = (0..500)
        .map(|i| ExportTickData {
            tick: i as u64,
            mode: 0,
            confidence: if i % 2 == 0 { 100 } else { 99 },
            energy_norm: 1_000_000 + (i as i64 % 10),
            rollback_count: 0,
            learning_delta: (i as i64 % 100),
            hashes: make_hash("noise"),
        })
        .collect();

    ExportReplayResult {
        ticks,
        trace_hash: "noise_floor".to_string(),
    }
}

pub fn attribution_conflict() -> ExportReplayResult {
    let mut ticks = Vec::new();

    for i in 0..300 {
        let is_chaos = i >= 100 && i < 200;

        let confidence = if is_chaos {
            (100 - ((i - 100) / 2)) as u8
        } else {
            100
        };

        let energy_norm = if is_chaos {
            1_000_000 + ((i - 100) as i64 * 50_000)
        } else {
            1_000_000
        };

        let rollback_count = if is_chaos {
            ((i - 100) / 10) as u32
        } else {
            0
        };

        let learning_delta = if is_chaos {
            (i - 100) as i64 * 5_000
        } else {
            0
        };

        ticks.push(ExportTickData {
            tick: i as u64,
            mode: 0,
            confidence,
            energy_norm,
            rollback_count,
            learning_delta,
            hashes: make_hash("conflict"),
        });
    }

    ExportReplayResult {
        ticks,
        trace_hash: "attribution_conflict".to_string(),
    }
}

pub fn single_tick_divergence() -> (ExportReplayResult, ExportReplayResult) {
    let run_a: Vec<ExportTickData> = (0..1000)
        .map(|i| ExportTickData {
            tick: i as u64,
            mode: 0,
            confidence: 100,
            energy_norm: 1_000_000,
            rollback_count: 0,
            learning_delta: 0,
            hashes: make_hash("baseline"),
        })
        .collect();

    let mut run_b = run_a.clone();
    run_b[500].hashes.struct_hash = "ffffffffffffffffffffffffffffffff".to_string();

    (
        ExportReplayResult {
            ticks: run_a,
            trace_hash: "divergence_a".to_string(),
        },
        ExportReplayResult {
            ticks: run_b,
            trace_hash: "divergence_b".to_string(),
        },
    )
}

pub fn long_tail_divergence() -> (ExportReplayResult, ExportReplayResult) {
    let run_a: Vec<ExportTickData> = (0..2500)
        .map(|i| ExportTickData {
            tick: i as u64,
            mode: 0,
            confidence: 100,
            energy_norm: 1_000_000,
            rollback_count: 0,
            learning_delta: 0,
            hashes: make_hash("baseline"),
        })
        .collect();

    let mut run_b = run_a.clone();

    for i in 500..run_b.len() {
        let drift = ((i - 500) / 100) as u32;
        run_b[i].energy_norm += drift as i64 * 100;
        run_b[i].hashes.energy_hash =
            format!("drift_{:08x}", (i as u32).wrapping_add(drift));
    }

    (
        ExportReplayResult {
            ticks: run_a,
            trace_hash: "long_tail_a".to_string(),
        },
        ExportReplayResult {
            ticks: run_b,
            trace_hash: "long_tail_b".to_string(),
        },
    )
}

pub fn dual_divergence_regions() -> (ExportReplayResult, ExportReplayResult) {
    let run_a: Vec<ExportTickData> = (0..1500)
        .map(|i| ExportTickData {
            tick: i as u64,
            mode: 0,
            confidence: 100,
            energy_norm: 1_000_000,
            rollback_count: 0,
            learning_delta: 0,
            hashes: make_hash("baseline"),
        })
        .collect();

    let mut run_b = run_a.clone();

    for i in 500..800 {
        run_b[i].hashes.energy_hash = "divergent_1".to_string();
    }

    for i in 800..1200 {
        run_b[i] = run_a[i].clone();
    }

    for i in 1200..run_b.len() {
        run_b[i].hashes.topo_hash = "divergent_2".to_string();
    }

    (
        ExportReplayResult {
            ticks: run_a,
            trace_hash: "dual_div_a".to_string(),
        },
        ExportReplayResult {
            ticks: run_b,
            trace_hash: "dual_div_b".to_string(),
        },
    )
}
