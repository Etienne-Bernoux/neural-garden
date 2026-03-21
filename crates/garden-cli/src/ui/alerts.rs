// Panneau Alertes — derniers evenements marquants de la simulation.

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::snapshot::SimSnapshot;

/// Rendu du panneau alertes : 5 derniers highlights ou message par defaut.
pub fn render(frame: &mut Frame, area: Rect, snapshot: &SimSnapshot) {
    let block = Block::default()
        .title(" Alertes ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let lines: Vec<Line> = if snapshot.recent_highlights.is_empty() {
        vec![Line::from(Span::styled(
            "Aucun événement marquant",
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        // Afficher les 5 derniers highlights
        snapshot
            .recent_highlights
            .iter()
            .rev()
            .take(5)
            .map(|h| Line::from(Span::styled(h.as_str(), Style::default().fg(Color::Yellow))))
            .collect()
    };

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}
