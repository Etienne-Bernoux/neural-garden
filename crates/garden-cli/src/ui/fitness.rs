// Panneau Fitness — courbe de fitness, population, et infos de progression.

use std::collections::VecDeque;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Sparkline};
use ratatui::Frame;

use crate::snapshot::SimSnapshot;

/// Rendu du panneau fitness : infos, warning convergence, sparkline fitness, sparkline population.
pub fn render(frame: &mut Frame, area: Rect, snapshot: &SimSnapshot) {
    let block = Block::default()
        .title(" Fitness ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Infos textuelles sur 2 lignes
    let info = format!(
        "Best: {:.1} | Worst: {:.1} | Gen: {}\nTick: {} | {:.0} t/s | Année {}",
        snapshot.best_fitness,
        snapshot.worst_fitness,
        snapshot.generation,
        snapshot.tick,
        snapshot.ticks_per_second,
        snapshot.year
    );
    let info_paragraph = Paragraph::new(info).style(Style::default().fg(Color::White));

    // Detection de convergence : best et worst trop proches
    let spread = if snapshot.best_fitness > 0.0 {
        (snapshot.best_fitness - snapshot.worst_fitness) / snapshot.best_fitness
    } else {
        0.0
    };
    let show_warning = spread < 0.05 && snapshot.best_fitness > 0.0;

    // Normalisation fitness en f32 avant conversion u64 (variation relative)
    let fitness_data: Vec<u64> = normalize_f32_to_u64(&snapshot.fitness_history);

    // Normalisation population (variation relative)
    let pop_data: Vec<u64> = normalize_usize_to_u64(&snapshot.population_history);

    // Layout adaptatif : warning optionnel entre infos et sparklines
    let warning_height = if show_warning { 1 } else { 0 };
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),              // infos
            Constraint::Length(warning_height), // warning convergence
            Constraint::Percentage(60),         // sparkline fitness
            Constraint::Percentage(40),         // sparkline population
        ])
        .split(inner);

    frame.render_widget(info_paragraph, layout[0]);

    // Warning convergence en rouge
    if show_warning {
        let warning = Paragraph::new("⚠ Banque convergée — diversité génétique faible")
            .style(Style::default().fg(Color::Red));
        frame.render_widget(warning, layout[1]);
    }

    if !fitness_data.is_empty() {
        let sparkline = Sparkline::default()
            .data(&fitness_data)
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(sparkline, layout[2]);
    }

    if !pop_data.is_empty() {
        let sparkline = Sparkline::default()
            .data(&pop_data)
            .style(Style::default().fg(Color::Green));
        frame.render_widget(sparkline, layout[3]);
    }
}

/// Normalise un historique f32 en u64 [0..100] (variation relative dans la fenêtre).
fn normalize_f32_to_u64(history: &VecDeque<f32>) -> Vec<u64> {
    if history.is_empty() {
        return Vec::new();
    }
    let min_f = history.iter().copied().fold(f32::INFINITY, f32::min);
    let max_f = history.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let range_f = (max_f - min_f).max(0.01); // eviter division par zero
    history
        .iter()
        .map(|&v| ((v - min_f) / range_f * 100.0) as u64)
        .collect()
}

/// Normalise un historique usize en u64 [0..100] (variation relative dans la fenêtre).
fn normalize_usize_to_u64(history: &VecDeque<usize>) -> Vec<u64> {
    if history.is_empty() {
        return Vec::new();
    }
    let min_v = history.iter().copied().min().unwrap_or(0);
    let max_v = history.iter().copied().max().unwrap_or(1);
    let range = (max_v - min_v).max(1); // eviter division par zero
    history
        .iter()
        .map(|&v| ((v - min_v) * 100 / range) as u64)
        .collect()
}
