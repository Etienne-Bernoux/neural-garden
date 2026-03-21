// Panneau Cooperation — liens mycorhiziens, ratio symbiose, sparkline et tendance.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Sparkline};
use ratatui::Frame;

use crate::snapshot::SimSnapshot;

/// Rendu du panneau cooperation : liens, ratio, tendance et sparkline.
pub fn render(frame: &mut Frame, area: Rect, snapshot: &SimSnapshot) {
    let block = Block::default()
        .title(" Coopération ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Calcul du ratio liens/population
    let ratio = if snapshot.alive_count > 0 {
        snapshot.symbiosis_count as f64 / snapshot.alive_count as f64
    } else {
        0.0
    };

    // Calcul de la tendance : moyenne des 100 derniers vs 100 precedents
    let (trend_label, trend_color) = compute_trend(&snapshot.symbiosis_history);

    let lines = vec![
        Line::from(vec![
            Span::styled("Liens: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{}", snapshot.symbiosis_count),
                Style::default().fg(Color::Magenta),
            ),
            Span::styled(" | Ratio: ", Style::default().fg(Color::White)),
            Span::styled(format!("{:.2}", ratio), Style::default().fg(Color::Magenta)),
        ]),
        Line::from(vec![
            Span::styled("Tendance: ", Style::default().fg(Color::White)),
            Span::styled(trend_label, Style::default().fg(trend_color)),
        ]),
    ];

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(1)])
        .split(inner);

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, layout[0]);

    // Sparkline du nombre de liens dans le temps
    if !snapshot.symbiosis_history.is_empty() {
        let data: Vec<u64> = snapshot
            .symbiosis_history
            .iter()
            .map(|&v| v as u64)
            .collect();
        let sparkline = Sparkline::default()
            .data(&data)
            .style(Style::default().fg(Color::Magenta));
        frame.render_widget(sparkline, layout[1]);
    }
}

/// Calcule la tendance de symbiose a partir de l'historique.
/// Compare la moyenne des 100 derniers ticks vs les 100 precedents.
fn compute_trend(history: &std::collections::VecDeque<usize>) -> (String, Color) {
    let len = history.len();
    if len < 200 {
        return ("→ Pas assez de données".to_string(), Color::DarkGray);
    }

    // 100 derniers ticks
    let recent: f64 = history
        .iter()
        .skip(len - 100)
        .map(|&v| v as f64)
        .sum::<f64>()
        / 100.0;

    // 100 ticks precedents
    let previous: f64 = history
        .iter()
        .skip(len - 200)
        .take(100)
        .map(|&v| v as f64)
        .sum::<f64>()
        / 100.0;

    if previous < f64::EPSILON {
        // Eviter division par zero
        if recent > 0.0 {
            return ("↗ Coopération en hausse".to_string(), Color::Green);
        }
        return ("→ Stable".to_string(), Color::Yellow);
    }

    let change = (recent - previous) / previous;

    if change > 0.10 {
        ("↗ Coopération en hausse".to_string(), Color::Green)
    } else if change < -0.10 {
        ("↘ Coopération en baisse".to_string(), Color::Red)
    } else {
        ("→ Stable".to_string(), Color::Yellow)
    }
}
