// Deep dive Population — affichage plein ecran des metriques de population.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::snapshot::SimSnapshot;

/// Rendu du deep dive Population.
pub fn render(frame: &mut Frame, area: Rect, snapshot: &SimSnapshot) {
    let block = Block::default()
        .title(" Population ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Layout vertical : resume, ages, lignees
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // resume demographique
            Constraint::Length(8), // distribution des ages
            Constraint::Min(1),    // lignees
        ])
        .split(inner);

    render_summary_section(frame, sections[0], snapshot);
    render_age_section(frame, sections[1], snapshot);
    render_lineage_section(frame, sections[2], snapshot);
}

/// Resume demographique.
fn render_summary_section(frame: &mut Frame, area: Rect, snapshot: &SimSnapshot) {
    let lines = vec![
        Line::from(format!(
            "Plantes vivantes: {} | Lignées: {}",
            snapshot.alive_count, snapshot.lineage_count,
        )),
        Line::from(format!(
            "Naissances/an: {} | Morts/an: {}",
            snapshot.births_last_year, snapshot.deaths_last_year,
        )),
        Line::from(format!(
            "Carbon: {} | Nitrogen: {}",
            snapshot.carbon_count, snapshot.nitrogen_count,
        )),
        Line::from(format!("Âge moyen: {:.0} ticks", snapshot.average_age)),
    ];

    let block = Block::default()
        .title(" Démographie ")
        .borders(Borders::ALL);
    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}

/// Distribution des ages par tranches avec barres horizontales.
fn render_age_section(frame: &mut Frame, area: Rect, snapshot: &SimSnapshot) {
    let buckets = snapshot.age_buckets;
    let max_val = buckets.iter().copied().max().unwrap_or(1).max(1);
    let max_bar: u32 = 30;

    let labels = ["0-100:  ", "100-300:", "300-500:", "500+:   "];
    let lines: Vec<Line> = labels
        .iter()
        .zip(buckets.iter())
        .map(|(label, &count)| {
            let bar_len = (count * max_bar / max_val) as usize;
            Line::from(format!("{} {} {}", label, "█".repeat(bar_len), count,))
        })
        .collect();

    let block = Block::default()
        .title(" Distribution des âges ")
        .borders(Borders::ALL);
    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}

/// Tableau des lignees triees par taille decroissante.
fn render_lineage_section(frame: &mut Frame, area: Rect, snapshot: &SimSnapshot) {
    let mut sorted: Vec<(&u64, &usize)> = snapshot.lineage_distribution.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));

    let mut lines: Vec<Line> = Vec::with_capacity(sorted.len() + 2);
    lines.push(Line::from("Lignée  Plantes  Type"));
    lines.push(Line::from("─────────────────────────────"));

    // Afficher les 10 premieres lignees
    for (lineage_id, &count) in sorted.iter().take(10) {
        lines.push(Line::from(format!(
            "L{:<6} {:<8} (fitness moy: n/a)",
            lineage_id, count,
        )));
    }

    if sorted.len() > 10 {
        lines.push(Line::from(format!(
            "... et {} autres lignées",
            sorted.len() - 10,
        )));
    }

    if sorted.is_empty() {
        lines.push(Line::from("Aucune lignée active"));
    }

    let block = Block::default()
        .title(" Lignées (par taille) ")
        .borders(Borders::ALL);
    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}
