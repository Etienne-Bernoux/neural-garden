// Vue zoom d'un environnement : historique + champion + config.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use crate::nursery_snapshot::NurserySnapshot;

/// Rend la vue zoom d'un environnement.
pub fn render(frame: &mut Frame, snapshot: &NurserySnapshot, env_index: usize, area: Rect) {
    let Some(env) = snapshot.envs.get(env_index) else {
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Header
            Constraint::Min(5),    // Contenu principal
            Constraint::Length(1), // Shortcuts
        ])
        .split(area);

    // Header
    let gen_limit = if snapshot.total_generations == u32::MAX {
        "inf".to_string()
    } else {
        snapshot.total_generations.to_string()
    };
    let header_text = format!(" {} — Gen {}/{}", env.name, env.current_gen, gen_limit,);
    let header = Paragraph::new(header_text).style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_widget(header, chunks[0]);

    // Split horizontal : historique (gauche) + details (droite)
    let content = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(chunks[1]);

    render_history(frame, env, content[0]);
    render_details(frame, env, content[1]);

    // Shortcuts
    let shortcuts = Line::from(vec![
        Span::styled(" [Esc]", Style::default().fg(Color::Cyan)),
        Span::raw("Retour "),
        Span::styled("[P]", Style::default().fg(Color::Cyan)),
        Span::raw("Pause "),
        Span::styled("[S]", Style::default().fg(Color::Cyan)),
        Span::raw("Save "),
        Span::styled("[Q]", Style::default().fg(Color::Cyan)),
        Span::raw("Quit"),
    ]);
    frame.render_widget(Paragraph::new(shortcuts), chunks[2]);
}

fn render_history(frame: &mut Frame, env: &crate::nursery_snapshot::EnvSnapshot, area: Rect) {
    // Historique des generations (plus recent en haut)
    let items: Vec<ListItem> = env
        .history
        .iter()
        .rev()
        .map(|entry| {
            ListItem::new(format!(
                "Gen {:4} | best: {:.2} | avg: {:.2} | worst: {:.2}",
                entry.generation, entry.best, entry.avg, entry.worst,
            ))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(" Historique ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White)),
    );
    frame.render_widget(list, area);
}

fn render_details(frame: &mut Frame, env: &crate::nursery_snapshot::EnvSnapshot, area: Rect) {
    // Split vertical : champion (haut) + config (bas)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    render_champion_detail(frame, env, chunks[0]);
    render_config(frame, env, chunks[1]);
}

fn render_champion_detail(
    frame: &mut Frame,
    env: &crate::nursery_snapshot::EnvSnapshot,
    area: Rect,
) {
    let mut lines = vec![Line::from(format!(" Fitness: {:.4}", env.best_fitness))
        .style(Style::default().fg(Color::Yellow))];

    if let Some(stats) = &env.champion_stats {
        lines.push(Line::from(""));
        lines.push(Line::from(format!(
            " Biomass max:     {}",
            stats.max_biomass
        )));
        lines.push(Line::from(format!(
            " Territoire max:  {}",
            stats.max_territory
        )));
        lines.push(Line::from(format!(
            " Graines:         {}",
            stats.seeds_produced
        )));
        lines.push(Line::from(format!(
            " Duree de vie:    {} ticks",
            stats.lifetime
        )));
        lines.push(Line::from(format!(
            " Symbiose:        {}",
            stats.symbiotic_connections
        )));
        lines.push(Line::from(format!(
            " Echanges CN:     {:.1}",
            stats.cn_exchanges
        )));
        lines.push(Line::from(format!(
            " Exsudats emis:   {:.1}",
            stats.exudates_emitted
        )));
        lines.push(Line::from(format!(
            " Sol enrichi:     {:.1}",
            stats.soil_enriched
        )));
        lines.push(Line::from(format!(
            " Sol appauvri:    {:.1}",
            stats.soil_depleted
        )));
    }

    if let Some(traits) = &env.champion_traits {
        lines.push(Line::from(""));
        lines.push(Line::from(format!(
            " Brain hidden:    {}",
            traits.hidden_size()
        )));
        lines.push(Line::from(format!(
            " Max size:        {}",
            traits.max_size()
        )));
        lines.push(Line::from(format!(
            " Ratio C/N:       {:.2}",
            traits.carbon_nitrogen_ratio()
        )));
        lines.push(Line::from(format!(
            " Exudate:         {:?}",
            traits.exudate_type()
        )));
        lines.push(Line::from(format!(
            " Vitalite:        {:.1}",
            traits.vitality_factor()
        )));
        lines.push(Line::from(format!(
            " Energie:         {:.1}",
            traits.energy_factor()
        )));
    }

    let block = Block::default()
        .title(" Champion ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let widget = Paragraph::new(lines).block(block);
    frame.render_widget(widget, area);
}

fn render_config(frame: &mut Frame, env: &crate::nursery_snapshot::EnvSnapshot, area: Rect) {
    let config = &env.bed_config;
    let lines = vec![
        Line::from(format!(
            " Grille: {}x{}",
            config.grid_size, config.grid_size
        )),
        Line::from(format!(
            " Sol initial: C={:.2} N={:.2} H={:.2}",
            config.initial_carbon, config.initial_nitrogen, config.initial_humidity,
        )),
        Line::from(format!(" Lumiere: {:.1}", config.light_level)),
        Line::from(format!(
            " Regen: C={:.4} N={:.4} H={:.4}",
            config.carbon_regen_rate, config.nitrogen_regen_rate, config.humidity_regen_rate,
        )),
        Line::from(format!(" Fixtures: {}", config.fixtures.len())),
        Line::from(format!(" Max ticks: {}", config.max_ticks)),
    ];

    let block = Block::default()
        .title(" Config environnement ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let widget = Paragraph::new(lines).block(block);
    frame.render_widget(widget, area);
}
