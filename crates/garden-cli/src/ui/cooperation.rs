// Panneau Cooperation — liens mycorhiziens et ratio symbiose.

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::snapshot::SimSnapshot;

/// Rendu du panneau cooperation : nombre de liens, ratio, statut.
pub fn render(frame: &mut Frame, area: Rect, snapshot: &SimSnapshot) {
    let block = Block::default()
        .title(" Coopération ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    // Calcul du ratio liens/population
    let ratio = if snapshot.alive_count > 0 {
        snapshot.symbiosis_count as f64 / snapshot.alive_count as f64
    } else {
        0.0
    };

    // Statut textuel
    let (status_text, status_color) = if snapshot.symbiosis_count > 0 {
        ("Symbiose active", Color::Green)
    } else {
        ("Pas de symbiose", Color::DarkGray)
    };

    let lines = vec![
        Line::from(vec![
            Span::styled("Liens mycorhiziens: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{}", snapshot.symbiosis_count),
                Style::default().fg(Color::Magenta),
            ),
        ]),
        Line::from(vec![
            Span::styled("Ratio liens/pop: ", Style::default().fg(Color::White)),
            Span::styled(format!("{:.2}", ratio), Style::default().fg(Color::Magenta)),
        ]),
        Line::from(""),
        Line::from(Span::styled(status_text, Style::default().fg(status_color))),
    ];

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}
