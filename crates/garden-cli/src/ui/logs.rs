// Deep dive Logs — affichage plein ecran des highlights recents (scrollable).

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::snapshot::SimSnapshot;

/// Rendu du deep dive Logs : liste des highlights recents en plein ecran.
pub fn render(frame: &mut Frame, area: Rect, snapshot: &SimSnapshot) {
    let block = Block::default()
        .title(" Logs ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    if snapshot.recent_highlights.is_empty() {
        let paragraph = Paragraph::new(Span::styled(
            "Aucun événement marquant",
            Style::default().fg(Color::DarkGray),
        ))
        .block(block);
        frame.render_widget(paragraph, area);
        return;
    }

    // Afficher les highlights du plus recent au plus ancien
    let lines: Vec<Line> = snapshot
        .recent_highlights
        .iter()
        .rev()
        .map(|h| {
            let style = highlight_style(h);
            Line::from(Span::styled(h.as_str(), style))
        })
        .collect();

    let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

/// Choisit une couleur selon le prefixe du highlight.
fn highlight_style(text: &str) -> Style {
    if text.contains("[!]") {
        Style::default().fg(Color::LightRed)
    } else if text.contains("[x]") {
        Style::default().fg(Color::Red)
    } else if text.contains("[+]") {
        Style::default().fg(Color::Green)
    } else if text.contains("[~]") {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::Yellow)
    }
}
