//! TIMELINE TUI: Pure projection-only execution observatory
//!
//! Renders:
//! - Timeline (mode/confidence/energy)
//! - Operator heatmap (proxy-derived intensity)
//! - Causal backtrace (temporal annotations leading to divergence)
//! - Tick inspector (per-tick forensics)
//!
//! CONSTRAINT: No kernel mutation, no feedback, no inference of "true state"
//! This is pure measurement geometry.

use crate::telemetry_reader::{load, ExportTickData};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Line},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};
use std::io;

pub fn run(path_a: &str, path_b: &str) {
    let run_a = load(path_a);
    let run_b = load(path_b);

    enable_raw_mode().unwrap();
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).unwrap();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut state = UiState::new(run_a, run_b);

    loop {
        terminal.draw(|f| draw_ui(f, &state)).unwrap();

        if let Event::Key(key) = event::read().unwrap() {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Down => state.scroll_down(),
                KeyCode::Up => state.scroll_up(),
                KeyCode::Right => state.zoom_in(),
                KeyCode::Left => state.zoom_out(),
                KeyCode::Char('j') => state.move_selection_down(),
                KeyCode::Char('k') => state.move_selection_up(),
                _ => {}
            }
        }
    }

    disable_raw_mode().unwrap();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).unwrap();
}

// ============================================================================
// STATE MANAGEMENT
// ============================================================================

struct UiState {
    a: Vec<ExportTickData>,
    b: Vec<ExportTickData>,
    offset: usize,
    window: usize,
    selected: usize,
}

impl UiState {
    fn new(a: Vec<ExportTickData>, b: Vec<ExportTickData>) -> Self {
        Self {
            a,
            b,
            offset: 0,
            window: 40,
            selected: 0,
        }
    }

    fn scroll_down(&mut self) {
        self.offset = (self.offset + 1).min(self.a.len().saturating_sub(self.window));
    }
    fn scroll_up(&mut self) {
        self.offset = self.offset.saturating_sub(1);
    }
    fn zoom_in(&mut self) {
        self.window = self.window.saturating_sub(5).max(10);
    }
    fn zoom_out(&mut self) {
        self.window = (self.window + 5).min(200);
    }
    fn move_selection_down(&mut self) {
        self.selected = (self.selected + 1).min(self.a.len().saturating_sub(1));
    }
    fn move_selection_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }
}

// ============================================================================
// DIVERGENCE & PROXY OPERATOR SIGNALS
// ============================================================================

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct DivergenceProfile {
    struct_delta: u8,
    energy_delta: u8,
    topo_delta: u8,
    memory_delta: u8,
}

#[derive(Debug, Clone)]
struct OperatorIntensity {
    lie: u8,     // Non-linear coupling (mode-dependent)
    diss: u8,    // Energy dissipation (energy decay)
    back: u8,    // Backreaction (energy constraint)
    ema: u8,     // Memory lag (confidence/rollback-dependent)
}

/// Hash distance: XOR of byte pairs (proxy for structural divergence)
/// Reserved for divergence intensity visualization in future UI enhancements
#[allow(dead_code)]
fn hash_distance(a: &str, b: &str) -> u8 {
    a.bytes()
        .zip(b.bytes())
        .map(|(x, y)| (x ^ y).count_ones() as u8)
        .sum::<u8>()
        .min(255)
}

/// Compute divergence profile between two ticks
/// Reserved for per-hash structural delta visualization in future UI enhancements
#[allow(dead_code)]
fn divergence_profile(a: &ExportTickData, b: &ExportTickData) -> DivergenceProfile {
    DivergenceProfile {
        struct_delta: hash_distance(&a.hashes.struct_hash, &b.hashes.struct_hash),
        energy_delta: hash_distance(&a.hashes.energy_hash, &b.hashes.energy_hash),
        topo_delta: hash_distance(&a.hashes.topo_hash, &b.hashes.topo_hash),
        memory_delta: hash_distance(&a.hashes.memory_hash, &b.hashes.memory_hash),
    }
}

/// Derive operator intensity proxy from observable signals only
///
/// CONSTRAINT: No kernel introspection. Only from:
/// - energy_norm delta
/// - confidence change
/// - mode transitions
/// - learning_delta
/// - rollback_count
/// - hash divergence patterns
fn operator_intensity(
    curr: &ExportTickData,
    prev: Option<&ExportTickData>,
) -> OperatorIntensity {
    // LIE (Lie bracket): Non-linear coupling manifests as mode transitions and energy swings
    let lie = if let Some(p) = prev {
        if p.mode != curr.mode {
            200 // Strong mode transition signal
        } else {
            (curr.energy_norm.abs() / 100_000_000).min(100) as u8
        }
    } else {
        50
    };

    // DISS (Dissipation): Energy decay, confidence stability
    let diss = if let Some(p) = prev {
        let energy_decay = if p.energy_norm > curr.energy_norm {
            ((p.energy_norm - curr.energy_norm).abs() / 100_000_000).min(100) as u8
        } else {
            0
        };
        let conf_stability = (100 - curr.confidence).min(100) as u8;
        (energy_decay + conf_stability) / 2
    } else {
        50
    };

    // BACK (Backreaction): Energy constraint maintenance, learning delta spikes
    let back = if prev.is_some() {
        let learning_intensity = (curr.learning_delta.abs() / 1_000).min(100) as u8;
        let energy_constraint = (curr.confidence).min(100) as u8;
        (learning_intensity + energy_constraint) / 2
    } else {
        50
    };

    // EMA (Memory): Lagged response, rollback events, confidence changes
    let ema = if let Some(p) = prev {
        let rollback_signal = curr.rollback_count * 50; // Each rollback = 50 units
        let conf_delta = (curr.confidence as i16 - p.confidence as i16).abs() as u8;
        ((rollback_signal as u8).saturating_add(conf_delta)).min(200) / 2
    } else {
        50
    };

    OperatorIntensity {
        lie: lie.min(200),
        diss: diss.min(200),
        back: back.min(200),
        ema: ema.min(200),
    }
}

/// Find first divergence tick (any hash differs)
fn first_divergence(a: &[ExportTickData], b: &[ExportTickData]) -> Option<usize> {
    for i in 0..a.len().min(b.len()) {
        if a[i].hashes.struct_hash != b[i].hashes.struct_hash
            || a[i].hashes.energy_hash != b[i].hashes.energy_hash
            || a[i].hashes.topo_hash != b[i].hashes.topo_hash
            || a[i].hashes.memory_hash != b[i].hashes.memory_hash
        {
            return Some(i);
        }
    }
    None
}

// ============================================================================
// RENDERING
// ============================================================================

fn draw_ui(f: &mut ratatui::Frame, state: &UiState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),   // header
            Constraint::Percentage(35), // timeline
            Constraint::Percentage(20), // operator heatmap
            Constraint::Percentage(25), // causal backtrace
            Constraint::Percentage(20), // tick inspector
        ])
        .split(f.size());

    draw_header(f, chunks[0], state);
    draw_timeline(f, chunks[1], state);
    draw_operator_heatmap(f, chunks[2], state);
    draw_causal_backtrace(f, chunks[3], state);
    draw_inspector(f, chunks[4], state);
}

fn draw_header(f: &mut ratatui::Frame, area: Rect, state: &UiState) {
    let div = first_divergence(&state.a, &state.b);
    let status = if div.is_some() {
        "✗ DIVERGENCE DETECTED"
    } else {
        "✓ IDENTICAL"
    };

    let header = Paragraph::new(vec![Line::from(Span::styled(
        format!(
            "TIMELINE OBSERVATORY | offset={} window={} | {} | divergence={:?}",
            state.offset, state.window, status, div
        ),
        Style::default()
            .fg(if div.is_some() { Color::Red } else { Color::Green })
            .add_modifier(Modifier::BOLD),
    ))])
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, area);
}

fn draw_timeline(f: &mut ratatui::Frame, area: Rect, state: &UiState) {
    let end = (state.offset + state.window).min(state.a.len().min(state.b.len()));
    let div = first_divergence(&state.a, &state.b);

    let mut lines: Vec<Line> = vec![];

    for i in state.offset..end {
        let a = &state.a[i];
        let b = &state.b.get(i).unwrap_or(a);

        let diverged = a.hashes.struct_hash != b.hashes.struct_hash
            || a.hashes.energy_hash != b.hashes.energy_hash;

        let mode_char = match a.mode {
            0 => "█",
            1 => "▓",
            2 => "░",
            3 => "─",
            _ => "?",
        };

        let energy_intensity = (a.energy_norm.abs() as f64).log10().max(0.0) / 3.0;
        let energy_bar = "▇".repeat((energy_intensity as usize).min(10));

        let mut spans = vec![
            Span::styled(format!("t={:05} ", a.tick), Style::default().fg(Color::White)),
            Span::styled(format!("M{}", mode_char), Style::default().fg(Color::Yellow)),
            Span::styled(format!(" C{:02} ", a.confidence), Style::default().fg(Color::Magenta)),
            Span::styled(format!("E{}", energy_bar), Style::default().fg(Color::Blue)),
        ];

        if diverged {
            spans.push(Span::styled(" ✗", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)));
        }

        if div.map(|d| d == i).unwrap_or(false) {
            spans.push(Span::styled(" ◄◄◄", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)));
        }

        lines.push(Line::from(spans));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Timeline: Mode / Confidence / Energy");
    let para = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    f.render_widget(para, area);
}

fn draw_operator_heatmap(f: &mut ratatui::Frame, area: Rect, state: &UiState) {
    let end = (state.offset + state.window).min(state.a.len().min(state.b.len()));
    let mut lines: Vec<Line> = vec![];

    // Operator legend
    lines.push(Line::from(vec![
        Span::styled("Operator Heatmap (proxy-derived):", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    ]));

    let mut lie_row = vec![Span::styled("LIE  ", Style::default().fg(Color::Yellow))];
    let mut diss_row = vec![Span::styled("DISS ", Style::default().fg(Color::Cyan))];
    let mut back_row = vec![Span::styled("BACK ", Style::default().fg(Color::Magenta))];
    let mut ema_row = vec![Span::styled("EMA  ", Style::default().fg(Color::Green))];

    for i in state.offset..end {
        let a = &state.a[i];
        let prev = if i > 0 { state.a.get(i - 1) } else { None };

        let intensity = operator_intensity(a, prev);

        let lie_char = intensity_to_char(intensity.lie);
        let diss_char = intensity_to_char(intensity.diss);
        let back_char = intensity_to_char(intensity.back);
        let ema_char = intensity_to_char(intensity.ema);

        lie_row.push(Span::raw(lie_char));
        diss_row.push(Span::raw(diss_char));
        back_row.push(Span::raw(back_char));
        ema_row.push(Span::raw(ema_char));
    }

    lines.push(Line::from(lie_row));
    lines.push(Line::from(diss_row));
    lines.push(Line::from(back_row));
    lines.push(Line::from(ema_row));

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Operator Intensity (LIE/DISS/BACK/EMA)");
    let para = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    f.render_widget(para, area);
}

fn draw_causal_backtrace(f: &mut ratatui::Frame, area: Rect, state: &UiState) {
    let div = first_divergence(&state.a, &state.b);
    let mut lines: Vec<Line> = vec![];

    if let Some(div_tick) = div {
        lines.push(Line::from(Span::styled(
            format!("✗ First divergence at tick {}", div_tick),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )));

        // Backtrace: show last 10 ticks leading to divergence
        let backtrace_start = div_tick.saturating_sub(10);
        for i in backtrace_start..=div_tick {
            if i >= state.a.len() {
                break;
            }
            let tick = &state.a[i];
            let prev = if i > 0 { state.a.get(i - 1) } else { None };

            let mut annotation = String::new();

            if let Some(p) = prev {
                if p.mode != tick.mode {
                    annotation.push_str("(mode↔) ");
                }
                if tick.confidence < p.confidence {
                    annotation.push_str("(conf↓) ");
                }
                if tick.energy_norm.abs() > p.energy_norm.abs() {
                    annotation.push_str("(energy↑) ");
                }
                if tick.rollback_count > p.rollback_count {
                    annotation.push_str("(rollback!) ");
                }
            }

            let marker = if i == div_tick { "✗" } else { " " };
            lines.push(Line::from(Span::raw(format!(
                "{} t={:05} {} L:{} C:{} E:{} {}",
                marker, tick.tick, annotation, tick.learning_delta, tick.confidence, tick.energy_norm,
                if i == div_tick { "← DIVERGENCE" } else { "" }
            ))));
        }
    } else {
        lines.push(Line::from(Span::styled(
            "✓ No divergence — traces are identical",
            Style::default().fg(Color::Green),
        )));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Causal Backtrace (temporal deltas)");
    let para = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    f.render_widget(para, area);
}

fn draw_inspector(f: &mut ratatui::Frame, area: Rect, state: &UiState) {
    if state.selected >= state.a.len() {
        return;
    }

    let tick = &state.a[state.selected];
    let tick_b = state.b.get(state.selected).unwrap_or(tick);
    let diverged = tick.hashes.struct_hash != tick_b.hashes.struct_hash
        || tick.hashes.energy_hash != tick_b.hashes.energy_hash;

    let lines = vec![
        Line::from(Span::styled(
            format!("Tick {}", tick.tick),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::raw(format!(
            "Mode: {} | Confidence: {} | Energy: {} | Rollback: {} | Learning: {}",
            tick.mode, tick.confidence, tick.energy_norm, tick.rollback_count, tick.learning_delta
        ))),
        Line::from(Span::raw(format!("Struct: {}", tick.hashes.struct_hash))),
        Line::from(Span::raw(format!("Energy: {}", tick.hashes.energy_hash))),
        Line::from(Span::raw(format!("Topo:   {}", tick.hashes.topo_hash))),
        Line::from(Span::raw(format!("Memory: {}", tick.hashes.memory_hash))),
        Line::from(Span::styled(
            format!("Status: {}", if diverged { "✗ DIVERGED" } else { "✓ INTACT" }),
            Style::default().fg(if diverged { Color::Red } else { Color::Green }),
        )),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Tick Inspector");
    let para = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    f.render_widget(para, area);
}

// ============================================================================
// HELPERS
// ============================================================================

/// Convert intensity [0..200] to Unicode bar character
fn intensity_to_char(intensity: u8) -> &'static str {
    match intensity {
        0..=20 => "░",
        21..=40 => "▒",
        41..=60 => "▓",
        61..=100 => "█",
        101..=150 => "▓",
        _ => "█",
    }
}
