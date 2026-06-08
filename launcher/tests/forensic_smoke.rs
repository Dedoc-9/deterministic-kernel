use deterministic_launcher::forensic::fixtures;
use deterministic_launcher::forensics::operator_contribution;

#[test]
fn test_stable_equilibrium_loads() {
    let trace = fixtures::stable_equilibrium();
    assert_eq!(trace.ticks.len(), 1000);
}

#[test]
fn test_stable_equilibrium_attribution() {
    let trace = fixtures::stable_equilibrium();
    for i in 100..500 {
        let contrib = operator_contribution(&trace.ticks, i);
        let sum = contrib.lie + contrib.diss + contrib.back + contrib.ema;
        assert!((sum - 1.0).abs() < 1e-4, "Attribution sum: {}", sum);
    }
}

#[test]
fn test_confidence_collapse_loads() {
    let trace = fixtures::confidence_collapse();
    assert!(!trace.ticks.is_empty());
}

#[test]
fn test_noise_floor_minimal_events() {
    let trace = fixtures::noise_floor_trace();
    assert_eq!(trace.ticks.len(), 500);
}

#[test]
fn test_single_tick_divergence() {
    let (run_a, run_b) = fixtures::single_tick_divergence();
    let mut found = false;
    for i in 0..run_a.ticks.len() {
        if run_a.ticks[i].hashes.struct_hash != run_b.ticks[i].hashes.struct_hash {
            assert_eq!(i, 500, "Divergence should be at tick 500");
            found = true;
            break;
        }
    }
    assert!(found, "Divergence not detected");
}
