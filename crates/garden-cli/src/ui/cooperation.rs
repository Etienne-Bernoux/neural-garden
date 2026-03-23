// Deep dive Cooperation — affichage plein ecran des metriques de cooperation.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::snapshot::SimSnapshot;

/// Rendu du deep dive Cooperation.
pub fn render(frame: &mut Frame, area: Rect, snapshot: &SimSnapshot) {
    let block = Block::default()
        .title(" Coopération ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Layout vertical : resume, echanges, tendance
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // resume cooperation
            Constraint::Length(6), // echanges
            Constraint::Min(1),    // tendance
        ])
        .split(inner);

    render_summary_section(frame, sections[0], snapshot);
    render_exchanges_section(frame, sections[1], snapshot);
    render_trend_section(frame, sections[2], snapshot);
}

/// Resume des liens de cooperation.
fn render_summary_section(frame: &mut Frame, area: Rect, snapshot: &SimSnapshot) {
    let solitaires = snapshot
        .alive_count
        .saturating_sub(snapshot.cooperators_count);
    let solitaires_pct = if snapshot.alive_count > 0 {
        (solitaires as f32 / snapshot.alive_count as f32) * 100.0
    } else {
        0.0
    };

    let lines = vec![
        Line::from(format!("Liens actifs: {}", snapshot.symbiosis_count)),
        Line::from(format!(
            "Coopérateurs: {}/{} ({:.0}%)",
            snapshot.cooperators_count,
            snapshot.alive_count,
            snapshot.cooperators_ratio * 100.0,
        )),
        Line::from(format!(
            "Solitaires: {}/{} ({:.0}%)",
            solitaires, snapshot.alive_count, solitaires_pct,
        )),
    ];

    let block = Block::default()
        .title(" Liens mycorhiziens ")
        .borders(Borders::ALL);
    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}

/// Section echanges cumules.
fn render_exchanges_section(frame: &mut Frame, area: Rect, snapshot: &SimSnapshot) {
    // Moyenne par tick sur les 2 dernieres annees (2 * 360 = 720 ticks)
    let avg_per_tick = if snapshot.tick > 0 {
        let window = snapshot.tick.min(720);
        if window > 0 {
            snapshot.total_exchanges_2y / window as f32
        } else {
            0.0
        }
    } else {
        0.0
    };

    let lines = vec![
        Line::from(format!(
            "Total (2 ans): {:.1} unités",
            snapshot.total_exchanges_2y,
        )),
        Line::from(format!("Moyenne/tick: {:.3}", avg_per_tick)),
    ];

    let block = Block::default().title(" Échanges ").borders(Borders::ALL);
    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}

/// Tendance de la cooperation basee sur l'historique symbiose.
fn render_trend_section(frame: &mut Frame, area: Rect, snapshot: &SimSnapshot) {
    let trend = compute_trend(&snapshot.symbiosis_history);
    let (trend_icon, trend_label) = match trend {
        Trend::Rising => ("↗", "En hausse"),
        Trend::Falling => ("↘", "En baisse"),
        Trend::Stable => ("→", "Stable"),
        Trend::Unknown => ("?", "Données insuffisantes"),
    };

    let lines = vec![
        Line::from(format!("Tendance: {} {}", trend_icon, trend_label)),
        Line::from(""),
        Line::from("(Détail des liens non disponible dans le snapshot)"),
    ];

    let block = Block::default().title(" Tendance ").borders(Borders::ALL);
    let paragraph = Paragraph::new(lines)
        .style(Style::default().fg(Color::White))
        .block(block);
    frame.render_widget(paragraph, area);
}

enum Trend {
    Rising,
    Falling,
    Stable,
    Unknown,
}

/// Calcule la tendance en comparant la moyenne recente vs ancienne.
fn compute_trend(history: &std::collections::VecDeque<usize>) -> Trend {
    if history.len() < 100 {
        return Trend::Unknown;
    }

    let mid = history.len() / 2;
    let old_avg: f64 = history.iter().take(mid).sum::<usize>() as f64 / mid as f64;
    let new_avg: f64 =
        history.iter().skip(mid).sum::<usize>() as f64 / (history.len() - mid) as f64;

    let diff = new_avg - old_avg;
    let threshold = old_avg * 0.1; // 10% de variation

    if diff > threshold {
        Trend::Rising
    } else if diff < -threshold {
        Trend::Falling
    } else {
        Trend::Stable
    }
}
