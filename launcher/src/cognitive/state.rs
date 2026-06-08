//! Cognitive UI state layer — pure UI intent, no forensic coupling

#[derive(Clone, Debug)]
pub struct GlobalPlayback {
    pub tick: usize,
    pub min: usize,
    pub max: usize,
}

#[derive(Clone, Debug)]
pub enum FocusMode {
    All,
    ClusterOnly(usize),
    DivergenceOnly,
}

/// Pure UI state — never mutates forensic data
#[derive(Clone, Debug)]
pub struct CognitiveState {
    pub selected_run: usize,
    pub selected_cluster: Option<usize>,
    pub focus_mode: FocusMode,
    pub comparison_mode: bool,
    pub global: GlobalPlayback,
}

impl Default for CognitiveState {
    fn default() -> Self {
        Self {
            selected_run: 0,
            selected_cluster: None,
            focus_mode: FocusMode::All,
            comparison_mode: false,
            global: GlobalPlayback {
                tick: 0,
                min: 0,
                max: 0,
            },
        }
    }
}
