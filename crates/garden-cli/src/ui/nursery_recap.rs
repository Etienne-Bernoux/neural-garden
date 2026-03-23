// Vue recap de la pepiniere : tableau des environnements + detail champion.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};

use crate::nursery_snapshot::NurserySnapshot;

/// Rend la vue recap de la pepiniere.
pub fn render(frame: &mut Frame, snapshot: &NurserySnapshot, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Header
            Constraint::Min(5),    // Table envs
            Constraint::Length(6), // Champion detail
            Constraint::Length(1), // Shortcuts
        ])
        .split(area);

    render_header(frame, snapshot, chunks[0]);
    render_table(frame, snapshot, chunks[1]);
    render_champion(frame, snapshot, chunks[2]);
    render_shortcuts(frame, chunks[3]);
}

fn render_header(frame: &mut Frame, snapshot: &NurserySnapshot, area: Rect) {
    let status = if snapshot.finished {
        "Termine"
    } else if snapshot.paused {
        "PAUSE"
    } else {
        "En cours"
    };

    let text = format!(
        " Pepiniere — Gen {}/{} | {} envs | Pop {} | Seed {} | {}",
        snapshot.max_gen(),
        snapshot.total_generations,
        snapshot.envs.len(),
        snapshot.population,
        snapshot.seed,
        status,
    );

    let style = if snapshot.paused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else if snapshot.finished {
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    };

    let header = Paragraph::new(text).style(style);
    frame.render_widget(header, area);
}

fn render_table(frame: &mut Frame, snapshot: &NurserySnapshot, area: Rect) {
    let header = Row::new(vec![
        Cell::from("  Env"),
        Cell::from("Best"),
        Cell::from("Avg"),
        Cell::from("Worst"),
        Cell::from("Delta"),
        Cell::from("Temps"),
    ])
    .style(
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    )
    .height(1);

    let rows: Vec<Row> = snapshot
        .envs
        .iter()
        .enumerate()
        .map(|(i, env)| {
            let selected = i == snapshot.selected_env;
            let prefix = if selected { "> " } else { "  " };

            // Couleur du delta
            let delta_style = if env.delta_best > 0.01 {
                Style::default().fg(Color::Green)
            } else if env.delta_best < -0.01 {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let delta_text = if env.delta_best.abs() < 0.01 {
                "  0.00".to_string()
            } else {
                format!("{:+.2}", env.delta_best)
            };

            let row = Row::new(vec![
                Cell::from(format!("{}{}", prefix, env.name)),
                Cell::from(format!("{:.2}", env.best_fitness)),
                Cell::from(format!("{:.2}", env.avg_fitness)),
                Cell::from(format!("{:.2}", env.worst_fitness)),
                Cell::from(delta_text).style(delta_style),
                Cell::from(format!("{:.1}s", env.elapsed_secs)),
            ]);

            if selected {
                row.style(Style::default().bg(Color::DarkGray).fg(Color::White))
            } else {
                row
            }
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Min(20),   // Env
            Constraint::Length(10), // Best
            Constraint::Length(10), // Avg
            Constraint::Length(10), // Worst
            Constraint::Length(8),  // Delta
            Constraint::Length(7),  // Temps
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(" Environnements ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White)),
    );

    frame.render_widget(table, area);
}

fn render_champion(frame: &mut Frame, snapshot: &NurserySnapshot, area: Rect) {
    let Some(env) = snapshot.envs.get(snapshot.selected_env) else {
        return;
    };

    let mut lines = vec![Line::from(format!(
        " Fitness: {:.4}  |  Gen: {}",
        env.best_fitness, env.current_gen
    ))
    .style(Style::default().fg(Color::Yellow))];

    if let Some(stats) = &env.champion_stats {
        lines.push(Line::from(format!(
            " Biomass: {}  |  Territoire: {}  |  Graines: {}  |  Symbiose: {}  |  CN: {:.1}",
            stats.max_biomass,
            stats.max_territory,
            stats.seeds_produced,
            stats.symbiotic_connections,
            stats.cn_exchanges,
        )));
    }

    if let Some(traits) = &env.champion_traits {
        lines.push(Line::from(format!(
            " Brain: h={}  |  MaxSize: {}  |  C/N: {:.2}  |  Exudate: {:?}  |  Vit: {:.1}  |  Nrj: {:.1}",
            traits.hidden_size(),
            traits.max_size(),
            traits.carbon_nitrogen_ratio(),
            traits.exudate_type(),
            traits.vitality_factor(),
            traits.energy_factor(),
        )));
    }

    let block = Block::default()
        .title(format!(" Champion — {} ", env.name))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let champion = Paragraph::new(lines).block(block);
    frame.render_widget(champion, area);
}

fn render_shortcuts(frame: &mut Frame, area: Rect) {
    let shortcuts = Line::from(vec![
        Span::styled(" [↑↓]", Style::default().fg(Color::Cyan)),
        Span::raw("Nav "),
        Span::styled("[Enter]", Style::default().fg(Color::Cyan)),
        Span::raw("Zoom "),
        Span::styled("[P]", Style::default().fg(Color::Cyan)),
        Span::raw("Pause "),
        Span::styled("[S]", Style::default().fg(Color::Cyan)),
        Span::raw("Save "),
        Span::styled("[Q]", Style::default().fg(Color::Cyan)),
        Span::raw("Quit"),
    ]);
    frame.render_widget(Paragraph::new(shortcuts), area);
}
