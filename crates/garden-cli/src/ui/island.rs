// Deep dive Ile — minimap plein ecran + stats globales + calques.

use garden_core::domain::world::GRID_SIZE;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::snapshot::SimSnapshot;

/// Noms des calques indexes par leur id (0-6).
const LAYER_NAMES: [&str; 7] = [
    "Plantes",
    "Carbone",
    "Azote",
    "Humidité",
    "Racines",
    "Canopée",
    "Footprint",
];

/// Touches associees a chaque calque.
const LAYER_KEYS: [&str; 7] = ["0", "A", "B", "C", "D", "E", "F"];

/// Couleurs associees a chaque calque (pour le rendu heatmap).
const LAYER_COLORS: [Color; 7] = [
    Color::Green,       // 0 = plantes (non utilise pour heatmap)
    Color::Yellow,      // 1 = carbone
    Color::Blue,        // 2 = azote
    Color::Cyan,        // 3 = humidite
    Color::Red,         // 4 = racines
    Color::Green,       // 5 = canopee
    Color::Magenta,     // 6 = footprint
];

/// Rendu du deep dive Ile : stats + calques + minimap/heatmap plein ecran.
pub fn render(frame: &mut Frame, area: Rect, snapshot: &SimSnapshot) {
    let block = Block::default()
        .title(" Île ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Layout vertical : stats (2 lignes) + calques (1 ligne) + minimap (reste) + legende (1 ligne)
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // stats globales
            Constraint::Length(1), // selecteur calques
            Constraint::Min(1),   // minimap ou heatmap
            Constraint::Length(1), // legende
        ])
        .split(inner);

    render_stats(frame, sections[0], snapshot);
    render_layer_selector(frame, sections[1], snapshot);

    // Afficher le heatmap si calque actif > 0 et donnees disponibles
    if snapshot.island_layer_id > 0 && !snapshot.island_layer_data.is_empty() {
        render_heatmap(frame, sections[2], snapshot);
    } else {
        render_minimap(frame, sections[2], snapshot);
    }

    render_legend(frame, sections[3], snapshot);
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

/// Selecteur de calques avec surbrillance du calque actif.
fn render_layer_selector(frame: &mut Frame, area: Rect, snapshot: &SimSnapshot) {
    let active = snapshot.active_layer as usize;
    let mut spans: Vec<Span> = Vec::new();
    spans.push(Span::raw("Calques: "));

    for (i, (name, key)) in LAYER_NAMES.iter().zip(LAYER_KEYS.iter()).enumerate() {
        if i > 0 {
            spans.push(Span::raw(" "));
        }
        let label = format!("[{}]{}", key, name);
        if i == active {
            // Calque actif en surbrillance
            spans.push(Span::styled(
                label,
                Style::default()
                    .fg(LAYER_COLORS[i])
                    .add_modifier(Modifier::BOLD | Modifier::REVERSED),
            ));
        } else {
            spans.push(Span::styled(
                label,
                Style::default().fg(Color::DarkGray),
            ));
        }
    }

    let paragraph = Paragraph::new(Line::from(spans));
    frame.render_widget(paragraph, area);
}

/// Rendu heatmap du calque actif a partir des donnees f32 pleine resolution.
fn render_heatmap(frame: &mut Frame, area: Rect, snapshot: &SimSnapshot) {
    let data = &snapshot.island_layer_data;
    let grid_size = GRID_SIZE as usize;

    // Verifier que les donnees ont la bonne taille
    if data.len() != grid_size * grid_size {
        let paragraph = Paragraph::new("(donnees calque invalides)")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(paragraph, area);
        return;
    }

    // Trouver min/max pour normalisation
    let mut min_val = f32::MAX;
    let mut max_val = f32::MIN;
    for &v in data {
        if v < min_val {
            min_val = v;
        }
        if v > max_val {
            max_val = v;
        }
    }
    let range = max_val - min_val;

    let avail_w = area.width as usize;
    let avail_h = area.height as usize;
    let color = LAYER_COLORS[snapshot.island_layer_id as usize];

    // Caracteres d'intensite
    let glyphs: [&str; 5] = [" ", "░", "▒", "▓", "█"];

    let lines: Vec<Line> = (0..avail_h)
        .map(|row| {
            let gy = row * grid_size / avail_h;
            let gy = gy.min(grid_size - 1);

            let spans: Vec<Span> = (0..avail_w)
                .map(|col| {
                    let gx = col * grid_size / avail_w;
                    let gx = gx.min(grid_size - 1);
                    let val = data[gy * grid_size + gx];

                    // Normaliser entre 0 et 1
                    let norm = if range > 0.0 {
                        (val - min_val) / range
                    } else if max_val > 0.0 {
                        1.0
                    } else {
                        0.0
                    };

                    // Choisir le glyphe selon l'intensite
                    let idx = (norm * 4.0).round() as usize;
                    let idx = idx.min(4);
                    Span::styled(glyphs[idx], Style::default().fg(color))
                })
                .collect();

            Line::from(spans)
        })
        .collect();

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

/// Legende en bas du panneau.
fn render_legend(frame: &mut Frame, area: Rect, snapshot: &SimSnapshot) {
    let line = if snapshot.island_layer_id > 0 && !snapshot.island_layer_data.is_empty() {
        let layer_name = LAYER_NAMES
            .get(snapshot.island_layer_id as usize)
            .copied()
            .unwrap_or("?");
        Line::from(vec![
            Span::styled(
                format!("{}: ", layer_name),
                Style::default()
                    .fg(LAYER_COLORS[snapshot.island_layer_id as usize])
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("░ faible  ", Style::default().fg(Color::DarkGray)),
            Span::styled("▒ moyen  ", Style::default().fg(Color::Gray)),
            Span::styled("▓ élevé  ", Style::default().fg(Color::White)),
            Span::styled("█ max", Style::default().fg(Color::White)),
        ])
    } else {
        Line::from(Span::styled(
            "Vue plantes — appuyer A-F pour activer un calque",
            Style::default().fg(Color::DarkGray),
        ))
    };

    let paragraph = Paragraph::new(line);
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
