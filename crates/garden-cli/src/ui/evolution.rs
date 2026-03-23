// Deep dive Evolution — affichage plein ecran des metriques d'evolution.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::snapshot::SimSnapshot;

/// Rendu du deep dive Evolution.
pub fn render(frame: &mut Frame, area: Rect, snapshot: &SimSnapshot) {
    let block = Block::default()
        .title(" Évolution ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Layout vertical : fitness, banque, distribution
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // fitness vivantes
            Constraint::Length(7), // banque de graines
            Constraint::Length(6), // distribution C/N
            Constraint::Min(1),    // age buckets
        ])
        .split(inner);

    render_fitness_section(frame, sections[0], snapshot);
    render_bank_section(frame, sections[1], snapshot);
    render_distribution_section(frame, sections[2], snapshot);
    render_age_section(frame, sections[3], snapshot);
}

/// Section fitness des plantes vivantes.
fn render_fitness_section(frame: &mut Frame, area: Rect, snapshot: &SimSnapshot) {
    // Proxy : on utilise best/worst de la banque comme approximation
    let avg = if snapshot.bank_best_fitness > 0.0 {
        (snapshot.bank_best_fitness + snapshot.bank_worst_fitness) / 2.0
    } else {
        0.0
    };

    let lines = vec![
        Line::from(format!(
            "Fitness vivantes (proxy banque): min {:.0} | moy {:.0} | max {:.0}",
            snapshot.bank_worst_fitness, avg, snapshot.bank_best_fitness,
        )),
        Line::from(""),
        Line::from(format!(
            "Génération: {} | Best fitness banque: {:.0}",
            snapshot.generation, snapshot.bank_best_fitness,
        )),
    ];

    let block = Block::default().title(" Fitness ").borders(Borders::ALL);
    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}

/// Section banque de graines.
fn render_bank_section(frame: &mut Frame, area: Rect, snapshot: &SimSnapshot) {
    let best_str = format_compact(snapshot.bank_best_fitness);
    let worst_str = format_compact(snapshot.bank_worst_fitness);

    let mut lines = vec![
        Line::from(format!(
            "Banque de graines: {}/100 ({} compartiments)",
            snapshot.bank_total_genomes, snapshot.bank_compartments,
        )),
        Line::from(format!(
            "Spread: {:.0}% | Best: {} | Worst: {}",
            snapshot.bank_spread * 100.0,
            best_str,
            worst_str,
        )),
        Line::from(""),
    ];

    // Top 5 genomes
    if snapshot.bank_top5.is_empty() {
        lines.push(Line::from("(Banque vide)"));
    } else {
        lines.push(Line::from(Span::styled(
            " #  Fitness     H.Size  Type       MaxSize",
            Style::default().fg(Color::DarkGray),
        )));
        for (i, (fitness, hs, extype, maxs)) in snapshot.bank_top5.iter().enumerate() {
            lines.push(Line::from(format!(
                " {}  {:<10.0}  {:<6}  {:<9}  {}",
                i + 1,
                fitness,
                hs,
                extype,
                maxs,
            )));
        }
    }

    let block = Block::default()
        .title(" Banque de graines ")
        .borders(Borders::ALL);
    let paragraph = Paragraph::new(lines)
        .style(Style::default().fg(Color::White))
        .block(block);
    frame.render_widget(paragraph, area);
}

/// Distribution Carbon vs Nitrogen.
fn render_distribution_section(frame: &mut Frame, area: Rect, snapshot: &SimSnapshot) {
    let total = snapshot.carbon_count + snapshot.nitrogen_count;
    let (c_pct, n_pct) = if total > 0 {
        (
            (snapshot.carbon_count as f32 / total as f32) * 100.0,
            (snapshot.nitrogen_count as f32 / total as f32) * 100.0,
        )
    } else {
        (0.0, 0.0)
    };

    // Barres proportionnelles sur 30 caracteres max
    let max_bar = 30;
    let c_bar = if total > 0 {
        snapshot.carbon_count * max_bar / total
    } else {
        0
    };
    let n_bar = if total > 0 {
        snapshot.nitrogen_count * max_bar / total
    } else {
        0
    };

    let lines = vec![
        Line::from(format!(
            "Carbon:   {} {} ({:.0}%)",
            "█".repeat(c_bar),
            snapshot.carbon_count,
            c_pct,
        )),
        Line::from(format!(
            "Nitrogen: {} {} ({:.0}%)",
            "█".repeat(n_bar),
            snapshot.nitrogen_count,
            n_pct,
        )),
    ];

    let block = Block::default()
        .title(" Distribution C/N ")
        .borders(Borders::ALL);
    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}

/// Distribution des ages par tranches.
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

/// Formate un nombre en notation compacte (ex: 610000 -> "610K").
fn format_compact(value: f32) -> String {
    if value >= 1_000_000.0 {
        format!("{:.1}M", value / 1_000_000.0)
    } else if value >= 1_000.0 {
        format!("{:.0}K", value / 1_000.0)
    } else {
        format!("{:.0}", value)
    }
}
