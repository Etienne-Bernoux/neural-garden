// Panneau Fitness — courbe de fitness et infos de progression.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Sparkline};
use ratatui::Frame;

use crate::snapshot::SimSnapshot;

/// Rendu du panneau fitness : sparkline + infos textuelles.
pub fn render(frame: &mut Frame, area: Rect, snapshot: &SimSnapshot) {
    let block = Block::default()
        .title(" Fitness ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Infos textuelles au-dessus de la sparkline
    let info = format!(
        "Best: {:.1} | Gen: {} | Tick: {} | {:.0} t/s",
        snapshot.best_fitness, snapshot.generation, snapshot.tick, snapshot.ticks_per_second
    );
    let info_paragraph = Paragraph::new(info).style(Style::default().fg(Color::White));

    // Sparkline de l'historique fitness
    let data: Vec<u64> = snapshot
        .fitness_history
        .iter()
        .map(|f| (*f * 10.0) as u64) // multiplier pour plus de resolution dans la sparkline
        .collect();

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(inner);

    frame.render_widget(info_paragraph, layout[0]);

    if !data.is_empty() {
        let sparkline = Sparkline::default()
            .data(&data)
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(sparkline, layout[1]);
    }
}
