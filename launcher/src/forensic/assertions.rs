//! Forensic validation assertions

use crate::forensics::OperatorContribution;
use crate::telemetry_reader::ExportTickData;

pub fn assert_attribution_valid(contrib: &OperatorContribution, context: &str) {
    let sum = contrib.lie + contrib.diss + contrib.back + contrib.ema;
    assert!((sum - 1.0).abs() < 1e-4, "[{}] Sum={}, expected 1.0", context, sum);
    assert!(contrib.lie >= 0.0 && contrib.lie <= 1.0, "[{}] LIE out of range", context);
    assert!(contrib.diss >= 0.0 && contrib.diss <= 1.0, "[{}] DISS out of range", context);
    assert!(contrib.back >= 0.0 && contrib.back <= 1.0, "[{}] BACK out of range", context);
    assert!(contrib.ema >= 0.0 && contrib.ema <= 1.0, "[{}] EMA out of range", context);
    assert!(!contrib.lie.is_nan() && !contrib.diss.is_nan() && !contrib.back.is_nan() && !contrib.ema.is_nan(), "[{}] NaN detected", context);
}

pub fn assert_no_self_loops(from: usize, to: usize, context: &str) {
    assert_ne!(from, to, "[{}] Self-loop: {} → {}", context, from, to);
}

pub fn assert_events_sorted(ticks: &[u64], context: &str) {
    for i in 1..ticks.len() {
        assert!(ticks[i-1] <= ticks[i], "[{}] Events out of order", context);
    }
}

pub fn assert_no_false_divergence(trace: &[ExportTickData], context: &str) {
    let trace_copy = trace.to_vec();
    for i in 0..trace.len() {
        assert_eq!(
            trace[i].hashes.struct_hash, trace_copy[i].hashes.struct_hash,
            "[{}] False divergence at {}", context, i
        );
    }
}
