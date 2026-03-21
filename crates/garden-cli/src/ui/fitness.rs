// Panneau Fitness — courbe de fitness, population, et infos de progression.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Sparkline};
use ratatui::Frame;

use crate::snapshot::SimSnapshot;

/// Rendu du panneau fitness : infos, sparkline fitness, sparkline population.
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

    // Sparkline de l'historique fitness
    let fitness_data: Vec<u64> = snapshot
        .fitness_history
        .iter()
        .map(|f| (*f * 10.0) as u64) // multiplier pour plus de resolution
        .collect();

    // Sparkline de l'historique population
    let pop_data: Vec<u64> = snapshot
        .population_history
        .iter()
        .map(|&p| p as u64)
        .collect();

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),      // 2 lignes d'infos
            Constraint::Percentage(60), // sparkline fitness
            Constraint::Percentage(40), // sparkline population
        ])
        .split(inner);

    frame.render_widget(info_paragraph, layout[0]);

    if !fitness_data.is_empty() {
        let sparkline = Sparkline::default()
            .data(&fitness_data)
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(sparkline, layout[1]);
    }

    if !pop_data.is_empty() {
        let sparkline = Sparkline::default()
            .data(&pop_data)
            .style(Style::default().fg(Color::Green));
        frame.render_widget(sparkline, layout[2]);
    }
}
