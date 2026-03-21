// Panneau Diversite — nombre de lignees, sante de la diversite, distribution.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Bar, BarChart, BarGroup, Block, Borders, Paragraph};
use ratatui::Frame;

use crate::snapshot::SimSnapshot;

/// Rendu du panneau diversite : lignees, indicateur de sante, barres de distribution.
pub fn render(frame: &mut Frame, area: Rect, snapshot: &SimSnapshot) {
    let block = Block::default()
        .title(" Diversité ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(inner);

    // Trier par taille decroissante
    let mut sorted: Vec<(u64, usize)> = snapshot
        .lineage_distribution
        .iter()
        .map(|(&k, &v)| (k, v))
        .collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));

    // Calcul du % de la lignee dominante
    let dominant_pct = if snapshot.alive_count > 0 {
        sorted
            .first()
            .map(|(_, count)| *count as f64 / snapshot.alive_count as f64 * 100.0)
            .unwrap_or(0.0)
    } else {
        0.0
    };

    // Indicateur de sante
    let (health_label, health_color) = if dominant_pct > 70.0 {
        ("Monoculture!", Color::Red)
    } else {
        ("Saine", Color::Green)
    };

    // Infos textuelles
    let info_lines = vec![
        Line::from(vec![
            Span::styled(
                format!("{} lignées vivantes", snapshot.lineage_count),
                Style::default().fg(Color::White),
            ),
            Span::styled(
                format!(" | {} plantes", snapshot.alive_count),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![Span::styled(
            format!("Lignée dominante: {:.0}%", dominant_pct),
            Style::default().fg(Color::White),
        )]),
        Line::from(vec![
            Span::styled("Diversité: ", Style::default().fg(Color::White)),
            Span::styled(health_label, Style::default().fg(health_color)),
        ]),
    ];

    let info_paragraph = Paragraph::new(info_lines);
    frame.render_widget(info_paragraph, layout[0]);

    // Distribution des lignees sous forme de barres (max 8)
    sorted.truncate(8);

    // Verifier si le BarChart est pertinent
    let max_count = sorted.iter().map(|(_, c)| *c).max().unwrap_or(0);

    if max_count <= 1 {
        // Pas de lignee dominante — message explicatif au lieu du BarChart
        let msg = Paragraph::new("Chaque plante est sa propre lignée\nPas de cluster évolutif")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(msg, layout[1]);
    } else if !sorted.is_empty() {
        // Nommer les lignees avec des lettres pour la lisibilite
        let letters = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H'];
        let bars: Vec<Bar> = sorted
            .iter()
            .enumerate()
            .map(|(i, (_, count))| {
                let label = if i < letters.len() {
                    format!("{} ({})", letters[i], count)
                } else {
                    format!("? ({})", count)
                };
                Bar::default()
                    .label(label.into())
                    .value(*count as u64)
                    .style(Style::default().fg(Color::Cyan))
            })
            .collect();

        let bar_chart = BarChart::default()
            .data(BarGroup::default().bars(&bars))
            .bar_width(7)
            .bar_gap(1)
            .value_style(Style::default().fg(Color::White));

        frame.render_widget(bar_chart, layout[1]);
    }
}
