// src/features.rs
// EMU-OPS: Complete Advanced Features Module

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant, SystemTime};

// ============================================================================
// 1. MULTI-STAGE ROLLBACK
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveStateConfig {
    pub save_interval_secs: u32,
    pub max_states: usize,
    pub compression_enabled: bool,
}

impl Default for SaveStateConfig {
    fn default() -> Self {
        Self { save_interval_secs: 30, max_states: 3, compression_enabled: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveState {
    pub timestamp: Instant,
    pub frame_number: u64,
    pub state_data: Vec<u8>,
    pub checksum: u64,
    pub is_stable: bool,
}

impl SaveState {
    pub fn new(frame_number: u64, state_data: Vec<u8>) -> Self {
        let checksum = Self::calculate_checksum(&state_data);
        Self { timestamp: Instant::now(), frame_number, state_data, checksum, is_stable: false }
    }
    fn calculate_checksum(data: &[u8]) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish()
    }
    pub fn verify_integrity(&self) -> bool {
        Self::calculate_checksum(&self.state_data) == self.checksum
    }
    pub fn mark_stable(&mut self) { self.is_stable = true; }
}

#[derive(Debug, Clone)]
pub struct RollingSaveStates {
    states: VecDeque<SaveState>,
    config: SaveStateConfig,
    frame_counter: u64,
    interval_frames: u64,
    last_save_frame: u64,
}

impl RollingSaveStates {
    pub fn new(config: SaveStateConfig) -> Self {
        let interval_frames = (config.save_interval_secs as u64) * 60;
        Self { states: VecDeque::with_capacity(config.max_states), config, frame_counter: 0, interval_frames, last_save_frame: 0 }
    }
    pub fn update_frame(&mut self, frame_number: u64) -> bool {
        self.frame_counter = frame_number;
        if frame_number - self.last_save_frame >= self.interval_frames {
            self.last_save_frame = frame_number;
            true
        } else { false }
    }
    pub fn push_state(&mut self, state: SaveState) {
        if self.states.len() >= self.config.max_states { self.states.pop_front(); }
        self.states.push_back(state);
    }
    pub fn get_stable_state(&self) -> Option<&SaveState> {
        self.states.iter().rev().find(|s| s.is_stable)
    }
    pub fn try_recovery(&mut self) -> Result<SaveState> {
        for i in 0..self.states.len() {
            let state = self.get_state_at(i).ok_or_else(|| anyhow!("No state"))?.clone();
            if state.verify_integrity() {
                log::info!("Recovery using savestate from frame {}", state.frame_number);
                return Ok(state);
            }
        }
        Err(anyhow!("All savestates failed"))
    }
    fn get_state_at(&self, index: usize) -> Option<&SaveState> {
        if index < self.states.len() { Some(&self.states[self.states.len() - 1 - index]) } else { None }
    }
    pub fn get_stats(&self) -> SaveStateStats {
        SaveStateStats {
            total_states: self.states.len(),
            stable_states: self.states.iter().filter(|s| s.is_stable).count(),
            total_memory: self.states.iter().map(|s| s.state_data.len()).sum(),
            oldest_frame: self.states.front().map(|s| s.frame_number),
            newest_frame: self.states.back().map(|s| s.frame_number),
        }
    }
    pub fn clear(&mut self) { self.states.clear(); self.last_save_frame = 0; }
}

#[derive(Debug, Clone, Serialize)]
pub struct SaveStateStats {
    pub total_states: usize,
    pub stable_states: usize,
    pub total_memory: usize,
    pub oldest_frame: Option<u64>,
    pub newest_frame: Option<u64>,
}

// ============================================================================
// 2. CRASH FINGERPRINTING
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintConfig {
    pub memory_window_secs: u64,
    pub skip_recovery_threshold: u32,
}

impl Default for FingerprintConfig {
    fn default() -> Self { Self { memory_window_secs: 300, skip_recovery_threshold: 3 } }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct CrashFingerprint {
    pub stack_hash: String,
    pub opcode_sequence: Vec<u8>,
    pub register_hash: String,
    pub crash_pc: u64,
}

impl CrashFingerprint {
    pub fn new(stack_trace: &[u64], opcodes: &[u8], register_state: &[u64], crash_pc: u64) -> Self {
        let stack_hash = Self::hash_data(&stack_trace.iter().take(10).flat_map(|a| a.to_le_bytes()).collect::<Vec<_>>());
        let register_hash = Self::hash_data(&register_state.iter().flat_map(|r| r.to_le_bytes()).collect::<Vec<_>>());
        let opcode_sequence = opcodes.iter().rev().take(10).copied().collect();
        Self { stack_hash, opcode_sequence, register_hash, crash_pc }
    }
    fn hash_data(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }
    pub fn identifier(&self) -> String {
        format!("CRASH_{:x}_{:02x?}", self.crash_pc, &self.opcode_sequence[..std::cmp::min(4, self.opcode_sequence.len())])
    }
}

#[derive(Debug, Clone)]
pub struct CrashMemory {
    crashes: HashMap<CrashFingerprint, (SystemTime, u32)>,
    config: FingerprintConfig,
}

impl CrashMemory {
    pub fn new(config: FingerprintConfig) -> Self { Self { crashes: HashMap::new(), config } }
    pub fn record_crash(&mut self, fingerprint: CrashFingerprint) -> bool {
        let now = SystemTime::now();
        if let Some((_, count)) = self.crashes.get_mut(&fingerprint) {
            *count += 1;
            log::warn!("Repeated crash: {} (#{})", fingerprint.identifier(), count);
            if *count >= self.config.skip_recovery_threshold && now.duration_since(*self.crashes.get(&fingerprint).unwrap().0).unwrap_or(Duration::MAX).as_secs() < self.config.memory_window_secs {
                return true;
            }
        } else {
            self.crashes.insert(fingerprint, (now, 1));
        }
        false
    }
}

// ============================================================================
// 3. ANOMALY DETECTOR
// ============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TelemetrySample {
    pub memory_allocation_rate: f32,
    pub cache_miss_rate: f32,
    pub frame_time_ms: f32,
    pub cpu_temp: f32,
    pub gpu_utilization: f32,
}

impl Default for TelemetrySample {
    fn default() -> Self { Self { memory_allocation_rate: 0.0, cache_miss_rate: 0.0, frame_time_ms: 16.67, cpu_temp: 45.0, gpu_utilization: 50.0 } }
}

#[derive(Debug, Clone)]
pub struct AnomalyDetector {
    window: VecDeque<TelemetrySample>,
    crash_probability: f32,
}

impl AnomalyDetector {
    pub fn new() -> Self { Self { window: VecDeque::with_capacity(60), crash_probability: 0.0 } }
    pub fn add_sample(&mut self, sample: TelemetrySample) {
        if self.window.len() >= 60 { self.window.pop_front(); }
        self.window.push_back(sample);
        self.update_crash_probability();
    }
    fn update_crash_probability(&mut self) {
        if self.window.len() < 5 { self.crash_probability = 0.0; return; }
        let memory = self.detect_memory_anomaly();
        let cache = self.detect_cache_spike();
        let variance = self.detect_frame_time_variance();
        let thermal = self.detect_thermal_anomaly();
        self.crash_probability = (memory * 0.25 + cache * 0.35 + variance * 0.25 + thermal * 0.15).min(1.0);
        if self.crash_probability > 0.7 { log::warn!("High crash probability: {:.2}%", self.crash_probability * 100.0); }
    }
    fn detect_memory_anomaly(&self) -> f32 {
        if self.window.len() < 10 { return 0.0; }
        let samples: Vec<f32> = self.window.iter().map(|s| s.memory_allocation_rate).collect();
        let mean = samples.iter().sum::<f32>() / samples.len() as f32;
        let variance = samples.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / samples.len() as f32;
        let std_dev = variance.sqrt();
        if std_dev == 0.0 { return 0.0; }
        let z_score = (samples[samples.len()-1] - mean).abs() / std_dev;
        (z_score / 3.0).min(1.0)
    }
    fn detect_cache_spike(&self) -> f32 {
        if self.window.len() < 10 { return 0.0; }
        let latest = self.window.back().unwrap().cache_miss_rate;
        let avg: f32 = self.window.iter().take(self.window.len()-1).map(|s| s.cache_miss_rate).sum::<f32>() / (self.window.len()-1) as f32;
        if avg == 0.0 { return 0.0; }
        let ratio = latest / avg;
        if ratio > 2.0 { 0.9 } else if ratio > 1.5 { 0.5 } else { 0.0 }
    }
    fn detect_frame_time_variance(&self) -> f32 {
        if self.window.len() < 5 { return 0.0; }
        let times: Vec<f32> = self.window.iter().map(|s| s.frame_time_ms).collect();
        let mean = times.iter().sum::<f32>() / times.len() as f32;
        let variance = times.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / times.len() as f32;
        let cv = variance.sqrt() / mean;
        (cv * 10.0).min(1.0)
    }
    fn detect_thermal_anomaly(&self) -> f32 {
        if let Some(latest) = self.window.back() {
            if latest.cpu_temp > 85.0 { 0.8 } else if latest.cpu_temp > 75.0 { 0.5 } else { 0.0 }
        } else { 0.0 }
    }
    pub fn predict_crash(&self) -> f32 { self.crash_probability }
    pub fn should_trigger_proactive_recovery(&self) -> bool { self.crash_probability > 0.7 }
}

impl Default for AnomalyDetector {
    fn default() -> Self { Self::new() }
}

// ============================================================================
// 4. SUBSYSTEM RECOVERY
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Subsystem { CpuCore, GpuEmulation, AudioMixer, InputHandler, MemoryController, IoController }

impl Subsystem {
    pub fn name(&self) -> &'static str {
        match self {
            Subsystem::CpuCore => "CPU Core",
            Subsystem::GpuEmulation => "GPU Emulation",
            Subsystem::AudioMixer => "Audio Mixer",
            Subsystem::InputHandler => "Input Handler",
            Subsystem::MemoryController => "Memory Controller",
            Subsystem::IoController => "I/O Controller",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubsystemHealth { Healthy, Degraded, Failed }

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RecoveryStrategy { FullRestart, StateRestore, Degrade, Isolate }

#[derive(Debug, Clone)]
pub struct SubsystemRecoveryManager {
    subsystems: HashMap<Subsystem, SubsystemState>,
    max_attempts: u32,
}

#[derive(Debug, Clone)]
struct SubsystemState {
    health: SubsystemHealth,
    error_count: u32,
}

impl SubsystemRecoveryManager {
    pub fn new() -> Self { Self { subsystems: HashMap::new(), max_attempts: 3 } }
    pub fn register(&mut self, subsystem: Subsystem) {
        self.subsystems.insert(subsystem, SubsystemState { health: SubsystemHealth::Healthy, error_count: 0 });
    }
    pub fn recover(&mut self, subsystem: Subsystem) -> Result<()> {
        let state = self.subsystems.get_mut(&subsystem).ok_or_else(|| anyhow!("Not found"))?;
        if state.error_count >= self.max_attempts { return Err(anyhow!("Max attempts")); }
        state.health = SubsystemHealth::Healthy;
        state.error_count = 0;
        Ok(())
    }
    pub fn health(&self, subsystem: Subsystem) -> Option<SubsystemHealth> {
        self.subsystems.get(&subsystem).map(|s| s.health)
    }
}

impl Default for SubsystemRecoveryManager {
    fn default() -> Self { Self::new() }
}

// ============================================================================
// 5. PERFORMANCE BUDGET
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceBudget { FixedFPS(u32), BatteryLife(u32), MaxQuality, Balanced }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfSettings {
    pub resolution_scale: f32,
    pub vsync: bool,
    pub threads: u32,
}

impl Default for PerfSettings {
    fn default() -> Self { Self { resolution_scale: 1.0, vsync: true, threads: 4 } }
}

#[derive(Debug, Clone)]
pub struct PerfBudgetMonitor {
    budget: PerformanceBudget,
    settings: PerfSettings,
    fps: f32,
}

impl PerfBudgetMonitor {
    pub fn new(budget: PerformanceBudget) -> Self {
        let mut s = PerfSettings::default();
        if let PerformanceBudget::FixedFPS(fps) = &budget {
            if *fps <= 30 { s.resolution_scale = 0.75; s.threads = 2; }
        }
        Self { budget, settings: s, fps: 60.0 }
    }
    pub fn update(&mut self, current_fps: f32) {
        self.fps = current_fps;
        match &self.budget {
            PerformanceBudget::FixedFPS(target) => {
                if current_fps < *target as f32 - 5.0 {
                    if self.settings.resolution_scale > 0.5 { self.settings.resolution_scale -= 0.1; }
                    else if self.settings.vsync { self.settings.vsync = false; }
                }
            }
            _ => {}
        }
    }
    pub fn settings(&self) -> &PerfSettings { &self.settings }
}

// ============================================================================
// 6. HEALTH DASHBOARD
// ============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HealthTelemetry {
    pub crash_prob: f32,
    pub frame_var: f32,
    pub cache_hit: f32,
    pub audio_glitches: u32,
    pub input_lag_ms: f32,
}

impl Default for HealthTelemetry {
    fn default() -> Self { Self { crash_prob: 0.0, frame_var: 2.0, cache_hit: 0.85, audio_glitches: 0, input_lag_ms: 10.0 } }
}

#[derive(Debug, Clone)]
pub struct HealthDashboard {
    telemetry: HealthTelemetry,
}

impl HealthDashboard {
    pub fn new() -> Self { Self { telemetry: HealthTelemetry::default() } }
    pub fn update(&mut self, t: HealthTelemetry) { self.telemetry = t; }
    pub fn score(&self) -> u32 {
        let crash = ((1.0 - self.telemetry.crash_prob) * 100.0) as u32;
        let var = if self.telemetry.frame_var <= 2.0 { 100 } else { ((20.0 - self.telemetry.frame_var.min(20.0)) / 18.0 * 100.0) as u32 };
        let cache = (self.telemetry.cache_hit * 100.0) as u32;
        let audio = if self.telemetry.audio_glitches >= 10 { 0 } else { ((10 - self.telemetry.audio_glitches) as f32 / 10.0 * 100.0) as u32 };
        let lag = if self.telemetry.input_lag_ms <= 10.0 { 100 } else if self.telemetry.input_lag_ms >= 50.0 { 0 } else { ((50.0 - self.telemetry.input_lag_ms) / 40.0 * 100.0) as u32 };
        (crash * 30 + var * 25 + cache * 20 + audio * 15 + lag * 10) / 100
    }
}

impl Default for HealthDashboard {
    fn default() -> Self { Self::new() }
}

// ============================================================================
// 7. SCENE DETECTION
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SceneType { Cutscene, Gameplay, Menu, Loading, Unknown }

#[derive(Debug, Clone, Copy, Default)]
pub struct SceneMetrics {
    pub input_rate: f32,
    pub gpu_util: f32,
    pub complexity: f32,
}

#[derive(Debug, Clone)]
pub struct SceneDetector {
    current: SceneType,
}

impl SceneDetector {
    pub fn new() -> Self { Self { current: SceneType::Unknown } }
    pub fn detect(&mut self, metrics: SceneMetrics) -> SceneType {
        let detected = if metrics.gpu_util < 10.0 && metrics.input_rate < 1.0 { SceneType::Loading }
        else if metrics.gpu_util > 70.0 && metrics.input_rate < 5.0 && metrics.complexity < 0.2 { SceneType::Cutscene }
        else if metrics.complexity < 0.3 && metrics.input_rate > 0.0 { SceneType::Menu }
        else if metrics.input_rate > 10.0 && metrics.complexity > 0.3 { SceneType::Gameplay }
        else { SceneType::Unknown };
        if detected != SceneType::Unknown { self.current = detected; }
        self.current
    }
    pub fn current(&self) -> SceneType { self.current }
}

impl Default for SceneDetector {
    fn default() -> Self { Self::new() }
}

// ============================================================================
// 8. DRY-RUN SIMULATION
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct DryRunEngine {
    replay: VecDeque<u64>, // simulated inputs
}

impl DryRunEngine {
    pub fn new() -> Self { Self { replay: VecDeque::with_capacity(60) } }
    pub fn record(&mut self, input: u64) {
        if self.replay.len() >= 60 { self.replay.pop_front(); }
        self.replay.push_back(input);
    }
    pub fn simulate(&self, res_scale: f32) -> (f32, bool) {
        let mut fps = 60.0;
        if res_scale < 1.0 { fps *= (2.0 - res_scale); }
        else if res_scale > 1.0 { fps /= res_scale; }
        let stable = fps > 45.0 && !self.replay.is_empty();
        (fps, stable)
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rollback() {
        let cfg = SaveStateConfig::default();
        let mut rm = RollingSaveStates::new(cfg);
        let state = SaveState::new(100, vec![1,2,3]);
        rm.push_state(state);
        assert_eq!(rm.get_stats().total_states, 1);
    }

    #[test]
    fn test_crash_memory() {
        let cfg = FingerprintConfig::default();
        let mut cm = CrashMemory::new(cfg);
        let fp = CrashFingerprint::new(&[0x1000], &[0x48], &[0], 0x4000);
        assert!(!cm.record_crash(fp.clone()));
        assert!(!cm.record_crash(fp.clone()));
        assert!(cm.record_crash(fp));
    }

    #[test]
    fn test_anomaly() {
        let mut ad = AnomalyDetector::new();
        for _ in 0..30 { ad.add_sample(TelemetrySample::default()); }
        assert!(ad.predict_crash() >= 0.0);
    }

    #[test]
    fn test_subsystem() {
        let mut sm = SubsystemRecoveryManager::new();
        sm.register(Subsystem::CpuCore);
        assert_eq!(sm.health(Subsystem::CpuCore), Some(SubsystemHealth::Healthy));
    }

    #[test]
    fn test_perf_budget() {
        let mut pb = PerfBudgetMonitor::new(PerformanceBudget::FixedFPS(60));
        pb.update(55.0);
        assert!(pb.settings().resolution_scale < 1.0);
    }

    #[test]
    fn test_health() {
        let mut hd = HealthDashboard::new();
        let score = hd.score();
        assert!(score <= 100);
    }

    #[test]
    fn test_scene() {
        let mut sd = SceneDetector::new();
        let metrics = SceneMetrics { input_rate: 20.0, gpu_util: 80.0, complexity: 0.8 };
        sd.detect(metrics);
        assert_eq!(sd.current(), SceneType::Gameplay);
    }

    #[test]
    fn test_dryrun() {
        let mut de = DryRunEngine::new();
        de.record(42);
        let (fps, stable) = de.simulate(1.0);
        assert!(fps > 0.0);
        assert!(stable);
    }
}
