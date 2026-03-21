// Panneau Diversite — nombre de lignees et distribution.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Bar, BarChart, BarGroup, Block, Borders, Paragraph};
use ratatui::Frame;

use crate::snapshot::SimSnapshot;

/// Rendu du panneau diversite : nombre de lignees, population, et barres de distribution.
pub fn render(frame: &mut Frame, area: Rect, snapshot: &SimSnapshot) {
    let block = Block::default()
        .title(" Diversité ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(1)])
        .split(inner);

    // Infos textuelles
    let info = format!(
        "Lignées: {} | Population: {}",
        snapshot.lineage_count, snapshot.alive_count
    );
    let info_paragraph = Paragraph::new(info).style(Style::default().fg(Color::White));
    frame.render_widget(info_paragraph, layout[0]);

    // Distribution des lignees sous forme de barres horizontales
    if !snapshot.lineage_distribution.is_empty() {
        // Trier par taille decroissante et limiter a 8 lignees max
        let mut sorted: Vec<(u64, usize)> = snapshot
            .lineage_distribution
            .iter()
            .map(|(&k, &v)| (k, v))
            .collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.truncate(8);

        let bars: Vec<Bar> = sorted
            .iter()
            .map(|(id, count)| {
                Bar::default()
                    .label(format!("L{}", id % 1000).into())
                    .value(*count as u64)
                    .style(Style::default().fg(Color::Cyan))
            })
            .collect();

        let bar_chart = BarChart::default()
            .data(BarGroup::default().bars(&bars))
            .bar_width(5)
            .bar_gap(1)
            .value_style(Style::default().fg(Color::White));

        frame.render_widget(bar_chart, layout[1]);
    }
}
