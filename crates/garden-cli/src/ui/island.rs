// Panneau Ile — population, saison, mini-carte.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::snapshot::SimSnapshot;

/// Rendu du panneau ile : stats + mini-carte Unicode.
pub fn render(frame: &mut Frame, area: Rect, snapshot: &SimSnapshot) {
    let block = Block::default()
        .title(" Île ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(1)])
        .split(inner);

    // Infos textuelles
    let season_str = format!("{:?}", snapshot.season);
    let pause_indicator = if snapshot.paused { " [PAUSE]" } else { "" };

    let info_lines = vec![
        Line::from(vec![
            Span::styled("Population: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{}", snapshot.alive_count),
                Style::default().fg(Color::Green),
            ),
            Span::styled(
                pause_indicator,
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Saison: ", Style::default().fg(Color::White)),
            Span::styled(
                &season_str,
                Style::default().fg(season_color(snapshot.season)),
            ),
            Span::styled(
                format!(" | Année: {}", snapshot.year),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("Biomasse: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{}", snapshot.total_biomass),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(vec![
            Span::styled("Âge moyen: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{:.1}", snapshot.average_age),
                Style::default().fg(Color::Cyan),
            ),
        ]),
    ];

    let info_paragraph = Paragraph::new(info_lines);
    frame.render_widget(info_paragraph, layout[0]);

    // Mini-carte Unicode
    render_minimap(frame, layout[1], &snapshot.minimap);
}

/// Couleur associee a chaque saison.
fn season_color(season: garden_core::application::season::Season) -> Color {
    use garden_core::application::season::Season;
    match season {
        Season::Spring => Color::Green,
        Season::Summer => Color::Yellow,
        Season::Autumn => Color::Red,
        Season::Winter => Color::Cyan,
    }
}

/// Rendu de la mini-carte en caracteres Unicode, adaptee a la taille du panneau.
/// ░ = mer, · = terre vide, ▪ = plante, █ = plante mature
fn render_minimap(frame: &mut Frame, area: Rect, minimap: &[Vec<u8>]) {
    if minimap.is_empty() {
        return;
    }

    let map_h = minimap.len();
    let map_w = minimap[0].len();
    let panel_h = area.height as usize;
    let panel_w = area.width as usize;

    // Echantillonner la minimap pour tenir dans le panneau
    let lines: Vec<Line> = (0..panel_h)
        .map(|py| {
            let my = py * map_h / panel_h.max(1);
            let spans: Vec<Span> = (0..panel_w)
                .map(|px| {
                    let mx = px * map_w / panel_w.max(1);
                    let cell = minimap
                        .get(my)
                        .and_then(|row| row.get(mx))
                        .copied()
                        .unwrap_or(0);
                    match cell {
                        0 => Span::styled("░", Style::default().fg(Color::DarkGray)),
                        1 => Span::styled("·", Style::default().fg(Color::Gray)),
                        2 => Span::styled("▪", Style::default().fg(Color::Green)),
                        3 => Span::styled("█", Style::default().fg(Color::LightGreen)),
                        _ => Span::styled(" ", Style::default()),
                    }
                })
                .collect();
            Line::from(spans)
        })
        .collect();

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}
