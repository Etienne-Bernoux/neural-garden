// Deep dive Ile — minimap plein ecran + stats globales + calques (futurs).

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::snapshot::SimSnapshot;

/// Rendu du deep dive Ile : stats + calques + minimap plein ecran.
pub fn render(frame: &mut Frame, area: Rect, snapshot: &SimSnapshot) {
    let block = Block::default()
        .title(" Île ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Layout vertical : stats (2 lignes) + calques (2 lignes) + minimap (reste)
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // stats globales
            Constraint::Length(2), // calques
            Constraint::Min(1),   // minimap
        ])
        .split(inner);

    render_stats(frame, sections[0], snapshot);
    render_layers_hint(frame, sections[1]);
    render_minimap(frame, sections[2], snapshot);
}

/// Stats globales de l'ile.
fn render_stats(frame: &mut Frame, area: Rect, snapshot: &SimSnapshot) {
    let lines = vec![
        Line::from(format!(
            "Cases vides: {} | Couverture: {:.0}%",
            snapshot.empty_land_cells,
            snapshot.land_coverage * 100.0,
        )),
        Line::from(format!(
            "Sol moyen: C={:.2} N={:.2}",
            snapshot.avg_soil_carbon, snapshot.avg_soil_nitrogen,
        )),
    ];
    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

/// Indication des calques futurs.
fn render_layers_hint(frame: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from(vec![
            Span::styled(
                "Calques: [A]Carbone [B]Azote [C]Racines [D]Canopée [E]Humidité [F]Footprint",
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(Span::styled(
            "(calques détaillés : prochaine version)",
            Style::default().fg(Color::DarkGray),
        )),
    ];
    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

/// Minimap plein ecran — echantillonne la grille simplifiee du snapshot.
/// 0=mer, 1=terre vide, 2=plante, 3=plante mature.
fn render_minimap(frame: &mut Frame, area: Rect, snapshot: &SimSnapshot) {
    if snapshot.minimap.is_empty() {
        let paragraph = Paragraph::new("(minimap non disponible)")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(paragraph, area);
        return;
    }

    let map_h = snapshot.minimap.len();
    let map_w = snapshot.minimap[0].len();

    // Espace disponible dans le widget
    let avail_w = area.width as usize;
    let avail_h = area.height as usize;

    // Echantillonnage : on mappe chaque caractere terminal sur une zone de la minimap.
    // Si la minimap est plus petite que la zone, on etire ; sinon on compresse.
    let lines: Vec<Line> = (0..avail_h)
        .map(|row| {
            let my = row * map_h / avail_h;
            let my = my.min(map_h - 1);

            let spans: Vec<Span> = (0..avail_w)
                .map(|col| {
                    let mx = col * map_w / avail_w;
                    let mx = mx.min(map_w - 1);
                    let cell = snapshot.minimap[my][mx];
                    cell_to_span(cell)
                })
                .collect();

            Line::from(spans)
        })
        .collect();

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

/// Convertit une valeur de cellule minimap en span colore.
fn cell_to_span(cell: u8) -> Span<'static> {
    match cell {
        0 => Span::styled("░", Style::default().fg(Color::Blue)),        // mer
        1 => Span::styled("·", Style::default().fg(Color::DarkGray)),    // terre vide
        2 => Span::styled("▪", Style::default().fg(Color::Green)),       // plante
        3 => Span::styled("▪", Style::default().fg(Color::LightGreen)),  // plante mature
        _ => Span::styled(" ", Style::default()),
    }
}
