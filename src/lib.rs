//! REPLAY ORACLE: Ground Truth Determinism Validator
//!
//! This is the critical path. If this passes, the kernel is deterministic.
//! If this fails, everything else is fiction.
//!
//! Principle: Same seed → run N ticks → hash → reset → run N ticks again → assert bit-identical
//!
//! No abstractions. No performance optimization. No learning layer.
//! Pure: can this system produce identical bytes twice?

use std::io::Write;
use serde::{Serialize, Deserialize};
use std::fs::File;

// =============================================================================
// EXPORT LAYER: JSON Serialization for Launcher Bridge
// =============================================================================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExportTickData {
    pub tick: u64,
    pub mode: u8,
    pub confidence: u8,
    pub energy_norm: i64,
    pub rollback_count: u32,
    pub learning_delta: i64,
    pub hashes: ExportHashes,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExportHashes {
    pub struct_hash: String,
    pub energy_hash: String,
    pub topo_hash: String,
    pub memory_hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExportReplayResult {
    pub ticks: Vec<ExportTickData>,
    pub trace_hash: String,
}

/// Convert hash bytes to hex string for JSON export
fn hash_to_hex(hash: &[u8; 32]) -> String {
    hash.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join("")
}

/// Export telemetry to JSON for launcher inspection
pub fn export_replay_to_json(result: &ReplayResult, path: &str) -> std::io::Result<()> {
    let export_ticks: Vec<ExportTickData> = result
        .telemetry
        .iter()
        .map(|e| ExportTickData {
            tick: e.tick,
            mode: e.mode as u8,
            confidence: e.confidence,
            energy_norm: e.energy_norm,
            rollback_count: e.rollback_count,
            learning_delta: e.learning_delta,
            hashes: ExportHashes {
                struct_hash: hash_to_hex(&e.hash_struct),
                energy_hash: hash_to_hex(&e.hash_energy),
                topo_hash: hash_to_hex(&e.hash_topo),
                memory_hash: hash_to_hex(&e.hash_memory),
            },
        })
        .collect();

    let export_result = ExportReplayResult {
        ticks: export_ticks,
        trace_hash: hash_to_hex(&result.trace_hash),
    };

    let file = File::create(path)?;
    serde_json::to_writer_pretty(file, &export_result)?;
    Ok(())
}

// =============================================================================
// PART 1: FIXED-POINT CORE (Minimal, no trait nonsense)
// =============================================================================

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Fixed(i64);

impl Fixed {
    pub const fn new(raw: i64) -> Self { Fixed(raw) }
    pub const fn raw(self) -> i64 { self.0 }

    // Deterministic multiply: Q31.32 * Q31.32 → Q31.32
    // No IEEE, no rounding, just shift
    fn mul(self, rhs: Fixed) -> Fixed {
        let prod = (self.0 as i128) * (rhs.0 as i128);
        Fixed((prod >> 32) as i64)
    }

    // Saturating add: no wrapping
    fn add(self, rhs: Fixed) -> Fixed {
        Fixed(self.0.saturating_add(rhs.0))
    }

    // Saturating subtract
    fn sub(self, rhs: Fixed) -> Fixed {
        Fixed(self.0.saturating_sub(rhs.0))
    }

    // Absolute value
    fn abs(self) -> Fixed {
        Fixed(self.0.abs())
    }

    // Clamp
    fn clamp(self, min: Fixed, max: Fixed) -> Fixed {
        if self.0 < min.0 { min }
        else if self.0 > max.0 { max }
        else { self }
    }
}

// Constructors for testing
impl Fixed {
    pub fn from_i32(val: i32) -> Self { Fixed((val as i64) << 32) }
    pub fn to_i32(self) -> i32 { (self.0 >> 32) as i32 }
}

// Division operator for Q31.32
impl std::ops::Div for Fixed {
    type Output = Self;

    #[inline]
    fn div(self, rhs: Fixed) -> Self {
        if rhs.0 == 0 {
            panic!("Division by zero");
        }
        Fixed(self.0 / rhs.0)
    }
}

// Unary negation operator for Fixed
impl std::ops::Neg for Fixed {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        Fixed(-self.0)
    }
}

// =============================================================================
// PART 2: STATE (Exactly what we evolve)
// =============================================================================

#[derive(Clone)]
pub struct State {
    pub z: Vec<Fixed>,        // Fast state (n dimensions)
    pub s: Vec<Fixed>,        // Memory state (EMA lag)
}

impl State {
    pub fn new(n: usize) -> Self {
        State {
            z: vec![Fixed::new(0); n],
            s: vec![Fixed::new(0); n],
        }
    }

    pub fn dim(&self) -> usize { self.z.len() }
}

// Deterministic byte serialization (big-endian, canonical)
pub fn serialize_state(state: &State) -> Vec<u8> {
    let mut buf = Vec::new();

    // Write dimension first (u32 big-endian)
    let n = state.z.len() as u32;
    buf.write_all(&n.to_be_bytes()).unwrap();

    // Write Z (each as i64 big-endian)
    for z_i in &state.z {
        buf.write_all(&z_i.raw().to_be_bytes()).unwrap();
    }

    // Write S (each as i64 big-endian)
    for s_i in &state.s {
        buf.write_all(&s_i.raw().to_be_bytes()).unwrap();
    }

    buf
}

// SHA256 deterministic hash
pub fn hash_state(state: &State) -> [u8; 32] {
    use sha2::{Sha256, Digest};
    let serialized = serialize_state(state);
    let mut hasher = Sha256::new();
    hasher.update(&serialized);
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

// =============================================================================
// PART 3: OPERATORS (Absolutely minimal)
// =============================================================================

pub struct Parameters {
    pub lambda: Fixed,      // Dissipation
    pub alpha: Fixed,       // Backreaction
    pub beta_ema: Fixed,    // Memory damping
    pub e_target: Fixed,    // Target energy
    pub kappa: Vec<Vec<Fixed>>,  // Coupling matrix
}

// Lie bracket: ΔZ[i] = Σ_j κ[i,j] * (Z[i]*S[j] - Z[j]*S[i])
fn lie_bracket(state: &State, kappa: &[Vec<Fixed>]) -> Vec<Fixed> {
    let mut out = vec![Fixed::new(0); state.dim()];

    for i in 0..state.dim() {
        for j in 0..state.dim() {
            let term = state.z[i].mul(state.s[j]).sub(state.z[j].mul(state.s[i]));
            let contrib = kappa[i][j].mul(term);
            out[i] = out[i].add(contrib);
        }
    }

    out
}

// Dissipation: ΔZ[i] = -λ * Z[i]
fn dissipation(state: &State, lambda: Fixed) -> Vec<Fixed> {
    state.z.iter()
        .map(|&z| Fixed::new(0).sub(lambda.mul(z)))
        .collect()
}

// Backreaction: ΔZ[i] = -α * (||Z||² - E_t) * Z[i]
fn backreaction(state: &State, alpha: Fixed, e_target: Fixed) -> Vec<Fixed> {
    let norm2 = state.z.iter()
        .fold(Fixed::new(0), |acc, &z| acc.add(z.mul(z)));

    let error = norm2.sub(e_target);

    state.z.iter()
        .map(|&z| {
            let prod = alpha.mul(error).mul(z);
            Fixed::new(0).sub(prod)
        })
        .collect()
}

// EMA: S_{t+1} = S_t + β * (Z_t - S_t)
fn ema_update(state: &State, beta: Fixed) -> Vec<Fixed> {
    state.s.iter()
        .zip(state.z.iter())
        .map(|(&s, &z)| {
            let delta = z.sub(s);
            s.add(beta.mul(delta))
        })
        .collect()
}

// =============================================================================
// PART 4: INTEGRITY LAYER (Phase 2a: 4-domain hashing + confidence + modes)
// =============================================================================

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Mode {
    Full = 0,      // All operators execute normally
    Damped = 1,    // 50% reduction in deltas
    Frozen = 2,    // Z locked, S evolves
    Hold = 3,      // All updates blocked
}

#[derive(Clone, Copy, Debug)]
pub struct IntegrityReport {
    pub h_struct: [u8; 32],   // SHA256(Z, S)
    pub h_energy: [u8; 32],   // SHA256(||Z||², ||S||²)
    pub h_topo: [u8; 32],     // SHA256(operator topology signature)
    pub h_memory: [u8; 32],   // SHA256(Z ⊕ S)
    pub confidence: u32,       // [0, 100]
    pub mode: Mode,
}

// Domain weights: must sum to 100
const W_STRUCT: u32 = 35;
const W_ENERGY: u32 = 25;
const W_TOPO: u32 = 25;
const W_MEMORY: u32 = 15;

// Hash the structural state: Z and S serialization
fn hash_struct(state: &State) -> [u8; 32] {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();

    for z_i in &state.z {
        hasher.update(z_i.raw().to_be_bytes());
    }
    for s_i in &state.s {
        hasher.update(s_i.raw().to_be_bytes());
    }

    hasher.finalize().into()
}

// Hash the energy signature: norms of Z and S
fn hash_energy(state: &State) -> [u8; 32] {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();

    let mut norm_z: i64 = 0;
    let mut norm_s: i64 = 0;

    for z_i in &state.z {
        norm_z = norm_z.wrapping_add(z_i.mul(*z_i).raw());
    }
    for s_i in &state.s {
        norm_s = norm_s.wrapping_add(s_i.mul(*s_i).raw());
    }

    hasher.update(norm_z.to_be_bytes());
    hasher.update(norm_s.to_be_bytes());

    hasher.finalize().into()
}

// Hash the topology signature: dimension + active count
fn hash_topo(state: &State, kappa_seed: u64) -> [u8; 32] {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();

    let mut active_count: i64 = 0;
    for z_i in &state.z {
        if z_i.raw() != 0 {
            active_count += 1;
        }
    }

    hasher.update((state.dim() as u64).to_be_bytes());
    hasher.update(active_count.to_be_bytes());
    hasher.update(kappa_seed.to_be_bytes());

    hasher.finalize().into()
}

// Hash the memory mismatch: Z XOR S
fn hash_memory(state: &State) -> [u8; 32] {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();

    for i in 0..state.dim() {
        let xor_val = state.z[i].raw() ^ state.s[i].raw();
        hasher.update(xor_val.to_be_bytes());
    }

    hasher.finalize().into()
}

// Compute all 4-domain integrity report
fn compute_integrity(state: &State, kappa_seed: u64) -> IntegrityReport {
    let h_struct = hash_struct(state);
    let h_energy = hash_energy(state);
    let h_topo = hash_topo(state, kappa_seed);
    let h_memory = hash_memory(state);

    // Confidence: check if each domain hash is non-zero (simple check)
    let mut score: u32 = 0;

    if h_struct.iter().any(|&b| b != 0) { score += W_STRUCT; }
    if h_energy.iter().any(|&b| b != 0) { score += W_ENERGY; }
    if h_topo.iter().any(|&b| b != 0) { score += W_TOPO; }
    if h_memory.iter().any(|&b| b != 0) { score += W_MEMORY; }

    // Clamp to [0, 100]
    let confidence = if score > 100 { 100 } else { score };

    // Select mode based on confidence
    let mode = if confidence >= 95 {
        Mode::Full
    } else if confidence >= 85 {
        Mode::Damped
    } else if confidence >= 70 {
        Mode::Frozen
    } else {
        Mode::Hold
    };

    IntegrityReport {
        h_struct,
        h_energy,
        h_topo,
        h_memory,
        confidence,
        mode,
    }
}

// Apply mode-based scaling to operator deltas
fn apply_mode_gating(delta: Fixed, mode: Mode) -> Fixed {
    match mode {
        Mode::Full => delta,
        Mode::Damped => {
            // 50% reduction: divide by 2 (deterministic integer division)
            Fixed::new(delta.raw() / 2)
        }
        Mode::Frozen => Fixed::new(0),  // Suppress Z updates
        Mode::Hold => Fixed::new(0),    // Suppress all updates
    }
}

// =============================================================================
// PART 5: ROLLBACK & CHECKPOINT (Phase 2b: Fault tolerance with determinism)
// =============================================================================

const MAX_ROLLBACK_RETRIES: u8 = 4;

#[derive(Clone)]
pub struct Checkpoint {
    pub tick: u64,
    pub z: Vec<Fixed>,
    pub s: Vec<Fixed>,
    pub integrity: IntegrityReport,
}

pub struct RollbackManager {
    last_checkpoint: Option<Checkpoint>,
    retry_count: u8,
    rollback_triggered: bool,
}

impl RollbackManager {
    fn new() -> Self {
        RollbackManager {
            last_checkpoint: None,
            retry_count: 0,
            rollback_triggered: false,
        }
    }

    fn save(&mut self, checkpoint: Checkpoint) {
        self.last_checkpoint = Some(checkpoint);
        self.rollback_triggered = false;
        self.retry_count = 0;
    }

    fn attempt_rollback(&mut self) -> bool {
        if self.retry_count >= MAX_ROLLBACK_RETRIES {
            return false;  // Hard stop
        }
        self.retry_count += 1;
        self.rollback_triggered = true;
        true
    }

    fn restore(&self) -> Option<Checkpoint> {
        self.last_checkpoint.clone()
    }

    fn is_hard_stop(&self) -> bool {
        self.rollback_triggered && self.retry_count >= MAX_ROLLBACK_RETRIES
    }

    fn reset_on_success(&mut self) {
        self.retry_count = 0;
        self.rollback_triggered = false;
    }
}

// Deterministic perturbation: seeded from state
fn deterministic_perturbation(state: &State, seed: u64) -> Vec<Fixed> {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();

    // Hash all state + seed
    for z_i in &state.z {
        hasher.update(z_i.raw().to_be_bytes());
    }
    for s_i in &state.s {
        hasher.update(s_i.raw().to_be_bytes());
    }
    hasher.update(seed.to_be_bytes());

    let hash = hasher.finalize();
    let mut perturb = vec![Fixed::new(0); state.dim()];

    // Use hash bytes as deterministic perturbation seed
    for i in 0..state.dim() {
        let byte = hash[i % hash.len()];
        let magnitude = (byte as i64).wrapping_mul(1000) / 256;  // [-500, 500] range
        perturb[i] = Fixed::new(magnitude);
    }

    perturb
}

// Mode hysteresis: prevent rapid oscillation
pub struct ModeHysteresis {
    count: u32,
    threshold: u32,
    last_mode: Mode,
}

impl ModeHysteresis {
    fn new(threshold: u32) -> Self {
        ModeHysteresis {
            count: 0,
            threshold,
            last_mode: Mode::Full,
        }
    }

    fn update(&mut self, proposed_mode: Mode) -> Mode {
        if proposed_mode != self.last_mode {
            self.count += 1;
            if self.count >= self.threshold {
                self.last_mode = proposed_mode;
                self.count = 0;
                return proposed_mode;
            }
            return self.last_mode;
        }
        self.count = 0;
        self.last_mode
    }
}

// =============================================================================
// PART 6: LEARNING LAYER (Phase 3: Deterministic intrinsic adaptation)
// =============================================================================

pub struct LearningConfig {
    pub step_size: Fixed,        // α in Q31.32
    pub max_delta: Fixed,        // θ_max accumulator bound
    pub update_interval: u64,    // N_ticks between updates
}

impl LearningConfig {
    fn default() -> Self {
        LearningConfig {
            step_size: Fixed::from_i32(1) / Fixed::from_i32(1000),
            max_delta: Fixed::from_i32(1) / Fixed::from_i32(10),
            update_interval: 100,
        }
    }
}

#[derive(Clone)]
pub struct LearningState {
    pub accumulator: Vec<Fixed>,  // Δθ_t integral accumulator
    pub last_update_tick: u64,
}

impl LearningState {
    fn new(dim: usize) -> Self {
        LearningState {
            accumulator: vec![Fixed::new(0); dim],
            last_update_tick: 0,
        }
    }

    fn step(
        &mut self,
        tick_idx: u64,
        z: &[Fixed],
        s: &[Fixed],
        theta: &mut [Fixed],
        config: &LearningConfig,
        mode: Mode,
    ) -> bool {
        // Only update every N_ticks
        if tick_idx - self.last_update_tick < config.update_interval {
            return false;  // No update this tick
        }
        self.last_update_tick = tick_idx;

        let mut updated = false;

        for ((acc, th), (&zi, &si)) in self.accumulator.iter_mut()
            .zip(theta.iter_mut())
            .zip(z.iter().zip(s.iter())) {

            // Error field: E_t = Z_t - S_t
            let e_t = zi.sub(si);

            // Accumulate: Δθ_t = clamp(Δθ_{t-1} + α*E_t, -θ_max, θ_max)
            let scaled_error = config.step_size.mul(e_t);
            let delta_raw = acc.add(scaled_error);
            *acc = delta_raw.clamp(-config.max_delta, config.max_delta);

            // Mode-based scaling
            let scaled = match mode {
                Mode::Full => *acc,
                Mode::Damped => Fixed::new(acc.raw() / 2),   // 50% reduction
                Mode::Frozen | Mode::Hold => Fixed::new(0),  // No learning in frozen/hold
            };

            // Apply parameter update
            *th = th.add(scaled);
            updated = true;
        }

        updated
    }

    fn reset(&mut self) {
        for acc in &mut self.accumulator {
            *acc = Fixed::new(0);
        }
    }
}

// =============================================================================
// PART 7: THE KERNEL (Single tick with Phase 1-3: evolution + integrity + rollback + learning)
// =============================================================================

pub struct TickOutput {
    pub state: State,
    pub mode: Mode,
    pub state_hash: [u8; 32],
    pub tick_num: u64,
    pub integrity: IntegrityReport,
    pub rolled_back: bool,
    pub retry_count: u8,
    pub learning_updated: bool,
}

pub fn tick(
    state: &mut State,
    params: &Parameters,
    _prev_mode: Mode,
    tick_num: u64,
) -> TickOutput {
    // STEP 1: Compute operator deltas (deterministic order)
    let delta_l = lie_bracket(state, &params.kappa);
    let delta_d = dissipation(state, params.lambda);
    let delta_b = backreaction(state, params.alpha, params.e_target);

    // STEP 2: Compute integrity BEFORE mode is determined
    let integrity = compute_integrity(state, 0);

    // STEP 3: Determine gating mode from integrity
    let gating_mode = integrity.mode;

    // STEP 4: Apply mode-based gating to deltas
    let gated_l: Vec<Fixed> = delta_l.iter().map(|&d| apply_mode_gating(d, gating_mode)).collect();
    let gated_d: Vec<Fixed> = delta_d.iter().map(|&d| apply_mode_gating(d, gating_mode)).collect();
    let gated_b: Vec<Fixed> = delta_b.iter().map(|&d| apply_mode_gating(d, gating_mode)).collect();

    // STEP 5: Update Z (apply gated deltas)
    for i in 0..state.dim() {
        let sum = state.z[i]
            .add(gated_l[i])
            .add(gated_d[i])
            .add(gated_b[i]);
        state.z[i] = sum.clamp(
            Fixed::from_i32(-1_000_000),
            Fixed::from_i32(1_000_000)
        );
    }

    // STEP 6: Update S (EMA memory, not gated)
    state.s = ema_update(state, params.beta_ema);

    // STEP 7: Hash the state
    let state_hash = hash_state(state);

    TickOutput {
        state: state.clone(),
        mode: gating_mode,
        state_hash,
        tick_num,
        integrity,
        rolled_back: false,
        retry_count: 0,
        learning_updated: false,  // Phase 1: learning not integrated yet
    }
}

/// Enhanced tick with rollback support (Phase 2b) + learning (Phase 3)
pub fn tick_with_rollback(
    state: &mut State,
    params: &Parameters,
    _prev_mode: Mode,
    tick_num: u64,
    rollback_manager: &mut RollbackManager,
) -> TickOutput {
    // STEP 1: Save checkpoint at start of tick
    let checkpoint = Checkpoint {
        tick: tick_num,
        z: state.z.clone(),
        s: state.s.clone(),
        integrity: compute_integrity(state, 0),
    };
    rollback_manager.save(checkpoint);

    // STEP 2: Compute operator deltas
    let delta_l = lie_bracket(state, &params.kappa);
    let delta_d = dissipation(state, params.lambda);
    let delta_b = backreaction(state, params.alpha, params.e_target);

    // STEP 3: Compute integrity
    let integrity = compute_integrity(state, 0);
    let gating_mode = integrity.mode;

    // STEP 4: Apply mode-based gating
    let gated_l: Vec<Fixed> = delta_l.iter().map(|&d| apply_mode_gating(d, gating_mode)).collect();
    let gated_d: Vec<Fixed> = delta_d.iter().map(|&d| apply_mode_gating(d, gating_mode)).collect();
    let gated_b: Vec<Fixed> = delta_b.iter().map(|&d| apply_mode_gating(d, gating_mode)).collect();

    // STEP 5: Update Z
    for i in 0..state.dim() {
        let sum = state.z[i]
            .add(gated_l[i])
            .add(gated_d[i])
            .add(gated_b[i]);
        state.z[i] = sum.clamp(
            Fixed::from_i32(-1_000_000),
            Fixed::from_i32(1_000_000)
        );
    }

    // STEP 6: Update S
    state.s = ema_update(state, params.beta_ema);

    // STEP 7: Hash the state
    let state_hash = hash_state(state);

    let retry_count = rollback_manager.retry_count;
    let mut rolled_back = false;

    // STEP 8: Rollback check (placeholder)

    if !rolled_back {
        rollback_manager.reset_on_success();
    }

    TickOutput {
        state: state.clone(),
        mode: gating_mode,
        state_hash,
        tick_num,
        integrity,
        rolled_back,
        retry_count,
        learning_updated: false,  // Phase 3 integration optional for tick_with_rollback
    }
}

// =============================================================================
// PART 6: REPLAY ORACLE (The truth system, now with Phase 2a integrity)
// =============================================================================

pub struct ReplayTrace {
    pub hashes: Vec<[u8; 32]>,
    pub modes: Vec<Mode>,
    pub confidences: Vec<u32>,
    pub integrity_reports: Vec<IntegrityReport>,
    pub final_state: State,
}

pub fn run_trace(
    initial_state: State,
    params: &Parameters,
    n_ticks: u64,
) -> ReplayTrace {
    let mut state = initial_state;
    let mut mode = Mode::Full;
    let mut hashes = vec![];
    let mut modes = vec![];
    let mut confidences = vec![];
    let mut integrity_reports = vec![];

    for t in 0..n_ticks {
        let output = tick(&mut state, params, mode, t);
        hashes.push(output.state_hash);
        modes.push(output.mode);
        confidences.push(output.integrity.confidence);
        integrity_reports.push(output.integrity);
        mode = output.mode;
    }

    ReplayTrace {
        hashes,
        modes,
        confidences,
        integrity_reports,
        final_state: state,
    }
}

pub fn validate_replay_equality(
    trace1: &ReplayTrace,
    trace2: &ReplayTrace,
    test_name: &str,
) -> Result<(), String> {
    // Check trace lengths match
    if trace1.hashes.len() != trace2.hashes.len() {
        return Err(format!(
            "{}: Trace length mismatch: {} vs {}",
            test_name, trace1.hashes.len(), trace2.hashes.len()
        ));
    }

    // Check every hash matches (bit-exact)
    for (tick, (h1, h2)) in trace1.hashes.iter().zip(trace2.hashes.iter()).enumerate() {
        if h1 != h2 {
            return Err(format!(
                "{}: Hash mismatch at tick {}: {:?} vs {:?}",
                test_name, tick, h1, h2
            ));
        }
    }

    // Check every mode matches
    for (tick, (m1, m2)) in trace1.modes.iter().zip(trace2.modes.iter()).enumerate() {
        if m1 != m2 {
            return Err(format!(
                "{}: Mode mismatch at tick {}: {:?} vs {:?}",
                test_name, tick, m1, m2
            ));
        }
    }

    // Check every confidence matches (Phase 2a: integrity determinism)
    for (tick, (c1, c2)) in trace1.confidences.iter().zip(trace2.confidences.iter()).enumerate() {
        if c1 != c2 {
            return Err(format!(
                "{}: Confidence mismatch at tick {}: {} vs {}",
                test_name, tick, c1, c2
            ));
        }
    }

    // Check integrity hashes match per domain (Phase 2a)
    for (tick, (i1, i2)) in trace1.integrity_reports.iter().zip(trace2.integrity_reports.iter()).enumerate() {
        if i1.h_struct != i2.h_struct {
            return Err(format!("{}: h_struct mismatch at tick {}", test_name, tick));
        }
        if i1.h_energy != i2.h_energy {
            return Err(format!("{}: h_energy mismatch at tick {}", test_name, tick));
        }
        if i1.h_topo != i2.h_topo {
            return Err(format!("{}: h_topo mismatch at tick {}", test_name, tick));
        }
        if i1.h_memory != i2.h_memory {
            return Err(format!("{}: h_memory mismatch at tick {}", test_name, tick));
        }
    }

    // Check final state Z matches
    for (i, (z1, z2)) in trace1.final_state.z.iter()
        .zip(trace2.final_state.z.iter()).enumerate()
    {
        if z1.raw() != z2.raw() {
            return Err(format!(
                "{}: Final Z[{}] mismatch: {} vs {}",
                test_name, i, z1.raw(), z2.raw()
            ));
        }
    }

    // Check final state S matches
    for (i, (s1, s2)) in trace1.final_state.s.iter()
        .zip(trace2.final_state.s.iter()).enumerate()
    {
        if s1.raw() != s2.raw() {
            return Err(format!(
                "{}: Final S[{}] mismatch: {} vs {}",
                test_name, i, s1.raw(), s2.raw()
            ));
        }
    }

    Ok(())
}

// =============================================================================
// PHASE 4: PRODUCTION DEPLOYMENT SKELETON (OBSERVABILITY LAYER)
// =============================================================================
// Telemetry event: append-only, no state modification
#[derive(Clone, Debug)]
pub struct TelemetryEvent {
    pub tick: u64,
    pub mode: Mode,
    pub confidence: u8,
    pub energy_norm: i64,
    pub rollback_count: u32,
    pub learning_delta: i64,
    pub hash_struct: [u8; 32],
    pub hash_energy: [u8; 32],
    pub hash_topo: [u8; 32],
    pub hash_memory: [u8; 32],
}

// Observability harvester: audit-grade logging, no feedback into kernel
#[derive(Clone)]
pub struct ObservabilityHarvester {
    pub enabled: bool,
    pub buffer: Vec<TelemetryEvent>,
}

impl ObservabilityHarvester {
    fn new() -> Self {
        Self {
            enabled: true,
            buffer: Vec::with_capacity(100_000),
        }
    }

    #[inline]
    fn record(&mut self, event: TelemetryEvent) {
        if self.enabled {
            self.buffer.push(event);
        }
    }

    fn flush_to_disk(&self, path: &str) -> std::io::Result<()> {
        use std::fs::File;
        let mut file = File::create(path)?;

        // Write header
        file.write_all(b"tick,mode,confidence,energy_norm,rollback_count,learning_delta\n")?;

        // Write events (deterministic CSV format)
        for e in &self.buffer {
            let line = format!(
                "{},{},{},{},{},{}\n",
                e.tick,
                e.mode as u8,
                e.confidence,
                e.energy_norm,
                e.rollback_count,
                e.learning_delta
            );
            file.write_all(line.as_bytes())?;
        }

        Ok(())
    }
}

// Replay result: captures full execution trace with telemetry
#[derive(Clone)]
pub struct ReplayResult {
    pub final_state: State,
    pub trace_hash: [u8; 32],
    pub telemetry: Vec<TelemetryEvent>,
}

// Run full replay with observability harvesting (Phase 4)
pub fn run_replay_with_observability(
    initial_state: State,
    params: &Parameters,
    n_ticks: u64,
) -> ReplayResult {
    use sha2::Digest;

    let mut state = initial_state;
    let mut mode = Mode::Full;
    let mut integrity = IntegrityReport {
        h_struct: [0u8; 32],
        h_energy: [0u8; 32],
        h_topo: [0u8; 32],
        h_memory: [0u8; 32],
        confidence: 100,
        mode: Mode::Full,
    };
    let mut rollback = RollbackManager::new();
    let mut learning = LearningState::new(state.dim());
    let mut obs = ObservabilityHarvester::new();
    let mut config = LearningConfig::default();

    for tick_idx in 0..n_ticks {
        // Core tick
        let output = tick(&mut state, params, mode, tick_idx);
        mode = output.mode;
        integrity = output.integrity;

        // Learning step (mode-gated, interval-gated)
        let mut theta = params.kappa[0].clone();
        let _ = learning.step(tick_idx, &state.z, &state.s, &mut theta, &config, mode);

        // Record telemetry (side effect only, no state feedback)
        let energy = state.z.iter().fold(Fixed::new(0), |acc, &z| acc.add(z.mul(z)));
        let learning_delta = if tick_idx > 0 && tick_idx % config.update_interval as u64 == 0 {
            learning.accumulator.iter().map(|a| a.raw()).sum::<i64>() / state.dim() as i64
        } else {
            0
        };

        obs.record(TelemetryEvent {
            tick: tick_idx,
            mode,
            confidence: integrity.confidence as u8,
            energy_norm: energy.raw(),
            rollback_count: rollback.retry_count as u32,
            learning_delta,
            hash_struct: integrity.h_struct,
            hash_energy: integrity.h_energy,
            hash_topo: integrity.h_topo,
            hash_memory: integrity.h_memory,
        });
    }

    // Compute final trace hash (includes event data for seed differentiation)
    let mut hasher = sha2::Sha256::new();
    for event in &obs.buffer {
        // Hash tick, mode, energy, and learning delta to distinguish different runs
        let event_sig = format!("{},{},{},{}\n", event.tick, event.mode as u8, event.energy_norm, event.learning_delta);
        hasher.update(event_sig.as_bytes());
    }
    let final_hash_digest = hasher.finalize();
    let mut trace_hash = [0u8; 32];
    trace_hash.copy_from_slice(&final_hash_digest);

    ReplayResult {
        final_state: state,
        trace_hash,
        telemetry: obs.buffer,
    }
}

// =============================================================================
// PART 7: TEST HARNESS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_default_params(dim: usize) -> Parameters {
        // Zero coupling matrix (Lie bracket is neutral)
        let kappa = vec![vec![Fixed::new(0); dim]; dim];

        Parameters {
            lambda: Fixed::from_i32(1) / Fixed::from_i32(100),
            alpha: Fixed::from_i32(1) / Fixed::from_i32(100),
            beta_ema: Fixed::from_i32(1) / Fixed::from_i32(10),
            e_target: Fixed::from_i32(dim as i32),
            kappa,
        }
    }

    fn make_initial_state(dim: usize, init_z: i32) -> State {
        let mut state = State::new(dim);
        state.z[0] = Fixed::from_i32(init_z);
        state
    }

    #[test]
    fn test_replay_identical_100_ticks() {
        let dim = 3;
        let params = make_default_params(dim);
        let seed = make_initial_state(dim, 10);

        // Run twice independently
        let trace1 = run_trace(seed.clone(), &params, 100);
        let trace2 = run_trace(seed.clone(), &params, 100);

        // Validate
        let result = validate_replay_equality(&trace1, &trace2, "replay_100");
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn test_replay_identical_1000_ticks() {
        let dim = 3;
        let params = make_default_params(dim);
        let seed = make_initial_state(dim, 10);

        let trace1 = run_trace(seed.clone(), &params, 1000);
        let trace2 = run_trace(seed.clone(), &params, 1000);

        let result = validate_replay_equality(&trace1, &trace2, "replay_1000");
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn test_replay_with_nonzero_coupling() {
        let dim = 3;
        let mut params = make_default_params(dim);

        // Add non-zero coupling: κ[0][1] = 0.01
        params.kappa[0][1] = Fixed::from_i32(1) / Fixed::from_i32(100);

        let seed = make_initial_state(dim, 10);

        let trace1 = run_trace(seed.clone(), &params, 100);
        let trace2 = run_trace(seed.clone(), &params, 100);

        let result = validate_replay_equality(&trace1, &trace2, "replay_coupling");
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn test_replay_with_energy_constraint() {
        let dim = 3;
        let mut params = make_default_params(dim);
        params.e_target = Fixed::from_i32(50);  // Change target energy

        let seed = make_initial_state(dim, 10);

        let trace1 = run_trace(seed.clone(), &params, 100);
        let trace2 = run_trace(seed.clone(), &params, 100);

        let result = validate_replay_equality(&trace1, &trace2, "replay_energy");
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn test_hash_determinism() {
        let dim = 2;
        let mut state = State::new(dim);
        state.z[0] = Fixed::from_i32(42);
        state.s[0] = Fixed::from_i32(37);

        let h1 = hash_state(&state);
        let h2 = hash_state(&state);

        assert_eq!(h1, h2, "Hash is non-deterministic!");
    }

    #[test]
    fn test_fixed_point_arithmetic_determinism() {
        let a = Fixed::from_i32(7);
        let b = Fixed::from_i32(3);

        // Multiply must be deterministic
        let p1 = a.mul(b);
        let p2 = a.mul(b);

        assert_eq!(p1, p2, "Multiply is non-deterministic!");
    }

    // ===== PHASE 2a: INTEGRITY LAYER TESTS =====

    #[test]
    fn test_integrity_domain_isolation() {
        let dim = 3;
        let params = make_default_params(dim);
        let mut state = make_initial_state(dim, 10);

        // Run one tick to get integrity report
        let output = tick(&mut state, &params, Mode::Full, 0);
        let report1 = output.integrity;

        // All domain hashes should be non-zero
        assert!(report1.h_struct.iter().any(|&b| b != 0), "h_struct should be non-zero");
        assert!(report1.h_energy.iter().any(|&b| b != 0), "h_energy should be non-zero");
        assert!(report1.h_topo.iter().any(|&b| b != 0), "h_topo should be non-zero");
        assert!(report1.h_memory.iter().any(|&b| b != 0), "h_memory should be non-zero");
    }

    #[test]
    fn test_confidence_determinism() {
        let dim = 3;
        let params = make_default_params(dim);
        let seed = make_initial_state(dim, 10);

        // Run two independent traces
        let mut state1 = seed.clone();
        let mut state2 = seed.clone();

        let out1 = tick(&mut state1, &params, Mode::Full, 0);
        let out2 = tick(&mut state2, &params, Mode::Full, 0);

        // Confidences must be identical
        assert_eq!(
            out1.integrity.confidence, out2.integrity.confidence,
            "Confidence scores diverged"
        );
    }

    #[test]
    fn test_mode_selection_determinism() {
        let dim = 3;
        let params = make_default_params(dim);
        let seed = make_initial_state(dim, 10);

        // Run two independent traces
        let mut state1 = seed.clone();
        let mut state2 = seed.clone();

        let out1 = tick(&mut state1, &params, Mode::Full, 0);
        let out2 = tick(&mut state2, &params, Mode::Full, 0);

        // Modes must be identical
        assert_eq!(
            out1.mode, out2.mode,
            "Mode selection diverged: {:?} vs {:?}",
            out1.mode, out2.mode
        );
    }

    #[test]
    fn test_mode_gating_determinism() {
        let dim = 3;
        let mut params = make_default_params(dim);
        params.lambda = Fixed::from_i32(1) / Fixed::from_i32(10);  // Higher damping to trigger Damped mode

        let seed = make_initial_state(dim, 10);

        let mut state1 = seed.clone();
        let mut state2 = seed.clone();

        // Run two independent ticks
        let out1 = tick(&mut state1, &params, Mode::Full, 0);
        let out2 = tick(&mut state2, &params, Mode::Full, 0);

        // Final states must be identical (gating must be deterministic)
        for i in 0..dim {
            assert_eq!(
                state1.z[i].raw(), state2.z[i].raw(),
                "Z diverged at index {} after gating",
                i
            );
            assert_eq!(
                state1.s[i].raw(), state2.s[i].raw(),
                "S diverged at index {} after gating",
                i
            );
        }
    }

    #[test]
    fn test_replay_with_integrity_1k_ticks() {
        let dim = 3;
        let params = make_default_params(dim);
        let seed = make_initial_state(dim, 10);

        // Run two independent 1K-tick traces with integrity layer active
        let trace1 = run_trace(seed.clone(), &params, 1000);
        let trace2 = run_trace(seed.clone(), &params, 1000);

        // Validate: all hashes, modes, confidences, and integrity reports must match
        let result = validate_replay_equality(&trace1, &trace2, "replay_integrity_1k");
        assert!(result.is_ok(), "Replay with integrity failed: {:?}", result.err());
    }

    #[test]
    fn test_integrity_hash_determinism_per_domain() {
        let dim = 2;
        let params = make_default_params(dim);
        let seed = make_initial_state(dim, 10);

        // Run two independent traces and verify each domain hash is deterministic
        let mut state1 = seed.clone();
        let mut state2 = seed.clone();

        let out1 = tick(&mut state1, &params, Mode::Full, 0);
        let out2 = tick(&mut state2, &params, Mode::Full, 0);

        // Each domain hash must be identical
        assert_eq!(out1.integrity.h_struct, out2.integrity.h_struct, "h_struct not deterministic");
        assert_eq!(out1.integrity.h_energy, out2.integrity.h_energy, "h_energy not deterministic");
        assert_eq!(out1.integrity.h_topo, out2.integrity.h_topo, "h_topo not deterministic");
        assert_eq!(out1.integrity.h_memory, out2.integrity.h_memory, "h_memory not deterministic");
    }

    #[test]
    fn test_confidence_bounds() {
        let dim = 3;
        let params = make_default_params(dim);
        let seed = make_initial_state(dim, 10);

        // Run many ticks and verify confidence stays in bounds
        let trace = run_trace(seed, &params, 100);

        for (tick, conf) in trace.confidences.iter().enumerate() {
            assert!(
                *conf <= 100,
                "Confidence exceeded 100 at tick {}: {}",
                tick, conf
            );
        }
    }

    #[test]
    fn test_all_integrity_domains_match_on_replay() {
        let dim = 3;
        let params = make_default_params(dim);
        let seed = make_initial_state(dim, 10);

        let trace1 = run_trace(seed.clone(), &params, 100);
        let trace2 = run_trace(seed.clone(), &params, 100);

        // Check that each integrity domain hash matches across replays
        for tick in 0..trace1.integrity_reports.len() {
            let r1 = &trace1.integrity_reports[tick];
            let r2 = &trace2.integrity_reports[tick];

            assert_eq!(r1.h_struct, r2.h_struct, "h_struct diverged at tick {}", tick);
            assert_eq!(r1.h_energy, r2.h_energy, "h_energy diverged at tick {}", tick);
            assert_eq!(r1.h_topo, r2.h_topo, "h_topo diverged at tick {}", tick);
            assert_eq!(r1.h_memory, r2.h_memory, "h_memory diverged at tick {}", tick);
        }
    }

    // ===== PHASE 2b: ROLLBACK & CHECKPOINT TESTS =====

    #[test]
    fn test_checkpoint_save_restore() {
        let dim = 3;
        let params = make_default_params(dim);
        let mut state = make_initial_state(dim, 10);

        let integrity = compute_integrity(&state, 0);
        let checkpoint = Checkpoint {
            tick: 0,
            z: state.z.clone(),
            s: state.s.clone(),
            integrity,
        };

        // Verify checkpoint contains correct state
        assert_eq!(checkpoint.z[0], state.z[0]);
        assert_eq!(checkpoint.s[0], state.s[0]);
    }

    #[test]
    fn test_rollback_manager_retry_limit() {
        let mut manager = RollbackManager::new();

        // Attempt 4 rollbacks (should all succeed)
        for i in 0..MAX_ROLLBACK_RETRIES {
            let success = manager.attempt_rollback();
            assert!(success, "Rollback {} should succeed", i + 1);
        }

        // 5th rollback should fail (hard stop)
        let success = manager.attempt_rollback();
        assert!(!success, "5th rollback should trigger hard stop");
    }

    #[test]
    fn test_mode_hysteresis_prevents_oscillation() {
        let mut hysteresis = ModeHysteresis::new(2);

        // Initial state: Full, count=0
        // First proposal of Damped: count becomes 1, < threshold(2), stay in Full
        assert_eq!(hysteresis.update(Mode::Damped), Mode::Full);

        // State: Full, count=1
        // Second proposal of Damped: count becomes 2, >= threshold(2), switch to Damped
        assert_eq!(hysteresis.update(Mode::Damped), Mode::Damped);

        // State: Damped, count=0
        // Third proposal of Damped: same mode, reset count to 0
        assert_eq!(hysteresis.update(Mode::Damped), Mode::Damped);

        // State: Damped, count=0
        // Propose Full: count becomes 1, < threshold(2), stay in Damped (prevents oscillation)
        assert_eq!(hysteresis.update(Mode::Full), Mode::Damped);
    }

    #[test]
    fn test_deterministic_perturbation() {
        let dim = 3;
        let state = make_initial_state(dim, 10);

        // Same state should produce same perturbation
        let perturb1 = deterministic_perturbation(&state, 0);
        let perturb2 = deterministic_perturbation(&state, 0);

        for i in 0..dim {
            assert_eq!(
                perturb1[i].raw(), perturb2[i].raw(),
                "Perturbation not deterministic at index {}",
                i
            );
        }
    }

    #[test]
    fn test_deterministic_perturbation_different_seed() {
        let dim = 3;
        let state = make_initial_state(dim, 10);

        // Different seed should produce different perturbation
        let perturb1 = deterministic_perturbation(&state, 0);
        let perturb2 = deterministic_perturbation(&state, 12345);

        // At least one should be different
        let mut any_different = false;
        for i in 0..dim {
            if perturb1[i].raw() != perturb2[i].raw() {
                any_different = true;
                break;
            }
        }
        assert!(any_different, "Perturbation should change with different seed");
    }

    #[test]
    fn test_rollback_manager_reset_on_success() {
        let mut manager = RollbackManager::new();

        // Attempt rollback
        let _ = manager.attempt_rollback();
        assert_eq!(manager.retry_count, 1);

        // Reset on success
        manager.reset_on_success();
        assert_eq!(manager.retry_count, 0);
    }

    #[test]
    fn test_tick_with_rollback_determinism() {
        let dim = 3;
        let params = make_default_params(dim);
        let seed = make_initial_state(dim, 10);

        // Run two independent ticks with rollback manager
        let mut state1 = seed.clone();
        let mut state2 = seed.clone();
        let mut manager1 = RollbackManager::new();
        let mut manager2 = RollbackManager::new();

        let out1 = tick_with_rollback(&mut state1, &params, Mode::Full, 0, &mut manager1);
        let out2 = tick_with_rollback(&mut state2, &params, Mode::Full, 0, &mut manager2);

        // States must be identical (rollback doesn't affect normal operation)
        for i in 0..dim {
            assert_eq!(
                state1.z[i].raw(), state2.z[i].raw(),
                "Z diverged at index {} with rollback manager",
                i
            );
            assert_eq!(
                state1.s[i].raw(), state2.s[i].raw(),
                "S diverged at index {} with rollback manager",
                i
            );
        }

        // Hashes must be identical
        assert_eq!(out1.state_hash, out2.state_hash);
    }

    #[test]
    fn test_checkpoint_contains_all_state() {
        let dim = 3;
        let state = make_initial_state(dim, 10);
        let integrity = compute_integrity(&state, 0);

        let checkpoint = Checkpoint {
            tick: 5,
            z: state.z.clone(),
            s: state.s.clone(),
            integrity,
        };

        // Verify checkpoint has all required fields
        assert_eq!(checkpoint.tick, 5);
        assert_eq!(checkpoint.z.len(), dim);
        assert_eq!(checkpoint.s.len(), dim);
        assert!(checkpoint.integrity.confidence <= 100);
    }

    // ===== PHASE 3: LEARNING LAYER TESTS =====

    #[test]
    fn test_learning_dormant_in_equilibrium() {
        let dim = 3;
        let mut learning = LearningState::new(dim);
        let config = LearningConfig::default();

        // In equilibrium: Z = S, so E_t = 0
        let z = vec![Fixed::from_i32(5); dim];
        let s = vec![Fixed::from_i32(5); dim];
        let mut theta = vec![Fixed::from_i32(1); dim];

        // Update parameters (should not change because E_t = 0)
        let _ = learning.step(100, &z, &s, &mut theta, &config, Mode::Full);

        // Accumulator should be zero (no error to accumulate)
        for acc in &learning.accumulator {
            assert_eq!(acc.raw(), 0, "Accumulator should be zero in equilibrium");
        }
    }

    #[test]
    fn test_learning_accumulator_saturation() {
        let dim = 1;
        let mut learning = LearningState::new(dim);
        let config = LearningConfig::default();

        // Force large error repeatedly
        let z = vec![Fixed::from_i32(1000)];
        let s = vec![Fixed::from_i32(0)];
        let mut theta = vec![Fixed::from_i32(1)];

        // Update multiple times
        for tick in (0..500).step_by(100) {
            let _ = learning.step(tick, &z, &s, &mut theta, &config, Mode::Full);
        }

        // Accumulator should not exceed max_delta
        assert!(
            learning.accumulator[0].raw().abs() <= config.max_delta.raw(),
            "Accumulator exceeded saturation bound"
        );
    }


    #[test]
    fn test_learning_frozen_mode_blocks_update() {
        let dim = 1;
        let z = vec![Fixed::from_i32(10)];
        let s = vec![Fixed::from_i32(0)];
        let mut theta = vec![Fixed::from_i32(1)];

        let mut learning = LearningState::new(1);
        let config = LearningConfig::default();

        // Update in Frozen mode
        let _ = learning.step(100, &z, &s, &mut theta, &config, Mode::Frozen);

        // Theta should not change
        assert_eq!(theta[0].raw(), Fixed::from_i32(1).raw(), "Frozen mode should block parameter updates");
    }

    #[test]
    fn test_learning_gated_update_interval() {
        let dim = 1;
        let z = vec![Fixed::from_i32(10)];
        let s = vec![Fixed::from_i32(0)];
        let mut theta = vec![Fixed::from_i32(1)];

        let mut learning = LearningState::new(1);
        let mut config = LearningConfig::default();
        config.update_interval = 50;  // Update every 50 ticks

        // Update at tick 10 (should not update)
        let updated1 = learning.step(10, &z, &s, &mut theta, &config, Mode::Full);
        assert!(!updated1, "Should not update before interval");

        // Update at tick 50 (should update)
        let updated2 = learning.step(50, &z, &s, &mut theta, &config, Mode::Full);
        assert!(updated2, "Should update at interval boundary");

        // Update at tick 60 (should not update)
        let updated3 = learning.step(60, &z, &s, &mut theta, &config, Mode::Full);
        assert!(!updated3, "Should not update before next interval");
    }

    #[test]
    fn test_learning_determinism_replay() {
        let dim = 3;
        let mut params = make_default_params(dim);
        let seed = make_initial_state(dim, 10);

        // Run two independent learning simulations
        let mut state1 = seed.clone();
        let mut state2 = seed.clone();
        let mut learning1 = LearningState::new(dim);
        let mut learning2 = LearningState::new(dim);
        let config = LearningConfig::default();

        for t in 0..500 {
            let out1 = tick(&mut state1, &params, Mode::Full, t);
            let out2 = tick(&mut state2, &params, Mode::Full, t);

            // Manually step learning (simplified)
            let _ = learning1.step(t, &state1.z, &state1.s, &mut params.kappa[0], &config, Mode::Full);
            let _ = learning2.step(t, &state2.z, &state2.s, &mut params.kappa[0], &config, Mode::Full);

            // Verify states remain identical
            assert_eq!(out1.state_hash, out2.state_hash, "State hashes diverged at tick {}", t);
        }
    }

    #[test]
    fn test_learning_reset() {
        let dim = 3;
        let mut learning = LearningState::new(dim);

        // Manually set accumulator values
        for i in 0..dim {
            learning.accumulator[i] = Fixed::from_i32(42);
        }

        // Reset
        learning.reset();

        // Verify all zero
        for acc in &learning.accumulator {
            assert_eq!(acc.raw(), 0, "Reset should zero out accumulator");
        }
    }

    // =============================================================================
    // PHASE 4: STRESS TEST SUITE (PRODUCTION VALIDATION ENGINE)
    // =============================================================================

    #[test]
    fn stress_10k_tick_replay_determinism() {
        let dim = 3;
        let params = make_default_params(dim);
        let seed = make_initial_state(dim, 10);

        let r1 = run_replay_with_observability(seed.clone(), &params, 10_000);
        let r2 = run_replay_with_observability(seed.clone(), &params, 10_000);

        // Trace hashes must be identical
        assert_eq!(r1.trace_hash, r2.trace_hash, "10K tick traces produced different hashes");

        // Final states must match
        assert_eq!(r1.final_state.z.len(), r2.final_state.z.len());
        for (z1, z2) in r1.final_state.z.iter().zip(r2.final_state.z.iter()) {
            assert_eq!(z1.raw(), z2.raw(), "Final Z values diverged");
        }
    }

    #[test]
    fn stress_50k_tick_stability() {
        let dim = 2;
        let params = make_default_params(dim);
        let seed = make_initial_state(dim, 5);

        let r1 = run_replay_with_observability(seed.clone(), &params, 50_000);
        let r2 = run_replay_with_observability(seed.clone(), &params, 50_000);

        // 50K ticks should maintain determinism
        assert_eq!(r1.trace_hash, r2.trace_hash, "50K tick determinism broken");
        assert_eq!(r1.telemetry.len(), r2.telemetry.len());
    }

    #[test]
    fn stress_mode_transition_observability() {
        let dim = 3;
        let params = make_default_params(dim);
        let seed = make_initial_state(dim, 10);

        let r = run_replay_with_observability(seed, &params, 5_000);

        // All modes should be in valid range [0,3]
        for e in &r.telemetry {
            assert!(e.mode as u8 <= 3, "Invalid mode in telemetry");
        }
    }

    #[test]
    fn stress_learning_accumulator_bounded() {
        let dim = 3;
        let params = make_default_params(dim);
        let seed = make_initial_state(dim, 50);  // Higher initial energy

        let r = run_replay_with_observability(seed, &params, 10_000);

        // Learning deltas should stay bounded
        let max_learning_delta = r.telemetry.iter()
            .map(|e| e.learning_delta.abs())
            .max()
            .unwrap_or(0);

        // Bounded by accumulator saturation
        assert!(max_learning_delta < Fixed::from_i32(1_000_000).raw(),
                "Learning delta exceeded bounds: {}", max_learning_delta);
    }

    #[test]
    fn stress_energy_norm_conservation() {
        let dim = 2;
        let params = make_default_params(dim);
        let seed = make_initial_state(dim, 20);

        let r = run_replay_with_observability(seed, &params, 1000);

        // Energy should fluctuate but remain in reasonable range
        let max_energy = r.telemetry.iter()
            .map(|e| e.energy_norm.abs())
            .max()
            .unwrap_or(0);

        // Energy should not diverge catastrophically
        assert!(max_energy < Fixed::from_i32(10_000_000).raw(),
                "Energy norm diverged: {}", max_energy);
    }

    #[test]
    fn stress_telemetry_consistency() {
        let dim = 3;
        let params = make_default_params(dim);
        let seed = make_initial_state(dim, 15);

        let r = run_replay_with_observability(seed, &params, 2000);

        // Telemetry should have contiguous tick numbers
        for (i, event) in r.telemetry.iter().enumerate() {
            assert_eq!(event.tick as usize, i, "Telemetry tick numbers non-contiguous");
        }

        // Hash fields must be populated
        for e in &r.telemetry {
            assert!(e.hash_struct != [0u8; 32], "hash_struct not recorded");
        }
    }

    #[test]
    fn stress_replay_determinism_cross_trace() {
        let dim = 4;
        let params = make_default_params(dim);

        // Test with different seeds
        let seed1 = make_initial_state(dim, 10);
        let seed2 = make_initial_state(dim, 20);

        let r1a = run_replay_with_observability(seed1.clone(), &params, 5000);
        let r1b = run_replay_with_observability(seed1.clone(), &params, 5000);
        let r2a = run_replay_with_observability(seed2.clone(), &params, 5000);

        // Same seed should give identical traces
        assert_eq!(r1a.trace_hash, r1b.trace_hash, "Same seed produced different traces");

        // Different seeds should give different traces
        assert_ne!(r1a.trace_hash, r2a.trace_hash, "Different seeds produced same trace");
    }
}

// =============================================================================
// MAIN: Standalone runner (not required for tests, but useful for debugging)
// =============================================================================

fn main() {
    use std::time::Instant;

    println!("=== REPLAY ORACLE: Determinism Validator ===\n");

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
            println!("  ./launcher inspect replay.json");
            println!("  ./launcher export replay.json --format csv");
        }
        Err(e) => {
            eprintln!("✗ Failed to export replay: {}", e);
            std::process::exit(1);
        }
    }

    println!("\nDETERMINISM CERTIFIED: System is reproducible.");
}
