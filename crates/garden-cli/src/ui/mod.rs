// Layout principal du TUI — barre d'etat, contenu (dashboard ou deep dive), raccourcis.

pub mod cooperation;
pub mod evolution;
pub mod island;
pub mod logs;
pub mod population;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::snapshot::SimSnapshot;

/// Mode d'affichage du TUI : dashboard compact ou deep dive sur un panneau.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppMode {
    Dashboard,
    Evolution,   // touche 1
    Population,  // touche 2
    Cooperation, // touche 3
    Island,      // touche 4
    Logs,        // touche 5
}

/// Rendu principal — dispatch selon le mode actif.
pub fn render(frame: &mut Frame, snapshot: &SimSnapshot, mode: AppMode) {
    let main_area = frame.area();

    // Layout vertical : barre d'etat (2 lignes) + contenu + raccourcis (1 ligne)
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // barre d'etat
            Constraint::Min(1),    // contenu
            Constraint::Length(1), // raccourcis
        ])
        .split(main_area);

    render_status_bar(frame, layout[0], snapshot);

    match mode {
        AppMode::Dashboard => render_dashboard(frame, layout[1], snapshot),
        AppMode::Evolution => evolution::render(frame, layout[1], snapshot),
        AppMode::Population => population::render(frame, layout[1], snapshot),
        AppMode::Cooperation => cooperation::render(frame, layout[1], snapshot),
        AppMode::Island => island::render(frame, layout[1], snapshot),
        AppMode::Logs => logs::render(frame, layout[1], snapshot),
    }

    render_shortcuts_bar(frame, layout[2], mode);
}

/// Barre d'etat en haut (2 lignes) — resume de la simulation.
fn render_status_bar(frame: &mut Frame, area: Rect, snapshot: &SimSnapshot) {
    let season_icon = match snapshot.season {
        garden_core::application::season::Season::Spring => "🌱 Printemps",
        garden_core::application::season::Season::Summer => "☀ Été",
        garden_core::application::season::Season::Autumn => "🍂 Automne",
        garden_core::application::season::Season::Winter => "❄ Hiver",
    };

    let pause_str = if snapshot.paused { " [PAUSE]" } else { "" };

    // Formater le best fitness de maniere lisible
    let best_str = format_compact(snapshot.bank_best_fitness);

    // Ligne 1
    let line1 = format!(
        "Tick {} | Année {} | {} | Pop: {} (C:{} N:{}) | Sym: {} | Best: {}",
        snapshot.tick,
        snapshot.year,
        season_icon,
        snapshot.alive_count,
        snapshot.carbon_count,
        snapshot.nitrogen_count,
        snapshot.symbiosis_count,
        best_str,
    );

    // Ligne 2
    let line2 = format!(
        "Births: {}/an | Deaths: {}/an | Bank: {} genomes ({} comp) | {:.0}K t/s{}",
        snapshot.births_last_year,
        snapshot.deaths_last_year,
        snapshot.bank_total_genomes,
        snapshot.bank_compartments,
        snapshot.ticks_per_second / 1000.0,
        pause_str,
    );

    let text = vec![
        Line::from(Span::styled(line1, Style::default().fg(Color::White))),
        Line::from(Span::styled(line2, Style::default().fg(Color::DarkGray))),
    ];

    let paragraph = Paragraph::new(text);
    frame.render_widget(paragraph, area);
}

/// Barre de raccourcis en bas (1 ligne) — le mode actif est en surbrillance.
fn render_shortcuts_bar(frame: &mut Frame, area: Rect, mode: AppMode) {
    let shortcuts = [
        ("1", "Évol", AppMode::Evolution),
        ("2", "Pop", AppMode::Population),
        ("3", "Coop", AppMode::Cooperation),
        ("4", "Île", AppMode::Island),
        ("5", "Logs", AppMode::Logs),
    ];

    let mut spans: Vec<Span> = Vec::new();

    for (key, label, item_mode) in &shortcuts {
        let style = if *item_mode == mode {
            Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        spans.push(Span::styled(format!("[{}]{} ", key, label), style));
    }

    // Raccourcis fixes
    spans.push(Span::styled(
        "[Espace]Pause ",
        Style::default().fg(Color::DarkGray),
    ));
    spans.push(Span::styled(
        "[Q]Quit ",
        Style::default().fg(Color::DarkGray),
    ));
    spans.push(Span::styled(
        "[S]Save ",
        Style::default().fg(Color::DarkGray),
    ));

    // Esc en surbrillance si on est en mode Dashboard
    let esc_style = if mode == AppMode::Dashboard {
        Style::default()
            .fg(Color::Black)
            .bg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    spans.push(Span::styled("[Esc]Dashboard", esc_style));

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, area);
}

/// Dashboard compact — resume condense de la simulation.
fn render_dashboard(frame: &mut Frame, area: Rect, snapshot: &SimSnapshot) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(6), Constraint::Min(1)])
        .split(area);

    // Bloc resume
    let fitness_min = snapshot.worst_fitness;
    let fitness_avg = if snapshot.best_fitness > 0.0 {
        (snapshot.best_fitness + snapshot.worst_fitness) / 2.0
    } else {
        0.0
    };
    let fitness_max = snapshot.best_fitness;

    let summary_lines = vec![
        Line::from(format!(
            "Population: {} ({} Carbon, {} Nitrogen)",
            snapshot.alive_count, snapshot.carbon_count, snapshot.nitrogen_count,
        )),
        Line::from(format!(
            "Lignées: {} | Coopérateurs: {:.0}%",
            snapshot.lineage_count,
            snapshot.cooperators_ratio * 100.0,
        )),
        Line::from(format!(
            "Fitness vivantes: min {:.0} / moy {:.0} / max {:.0}",
            fitness_min, fitness_avg, fitness_max,
        )),
        Line::from(format!(
            "Sol: C={:.2} N={:.2} | Couverture: {:.0}%",
            snapshot.avg_soil_carbon,
            snapshot.avg_soil_nitrogen,
            snapshot.land_coverage * 100.0,
        )),
        Line::from(format!(
            "Banque: {}/{} | Spread: {:.0}%",
            snapshot.bank_total_genomes,
            100, // capacite par defaut
            snapshot.bank_spread * 100.0,
        )),
    ];

    let summary_block = Block::default()
        .title(" Résumé ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));
    let summary = Paragraph::new(summary_lines).block(summary_block);
    frame.render_widget(summary, layout[0]);

    // Bloc alertes
    let alert_lines: Vec<Line> = if snapshot.recent_highlights.is_empty() {
        vec![Line::from(Span::styled(
            "Aucun événement marquant",
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        snapshot
            .recent_highlights
            .iter()
            .rev()
            .take(5)
            .map(|h| Line::from(Span::styled(h.as_str(), Style::default().fg(Color::Yellow))))
            .collect()
    };

    let alerts_block = Block::default()
        .title(" Alertes ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    let alerts = Paragraph::new(alert_lines).block(alerts_block);
    frame.render_widget(alerts, layout[1]);
}

/// Formate un nombre en notation compacte (ex: 610000 → "610K").
fn format_compact(value: f32) -> String {
    if value >= 1_000_000.0 {
        format!("{:.1}M", value / 1_000_000.0)
    } else if value >= 1_000.0 {
        format!("{:.0}K", value / 1_000.0)
    } else {
        format!("{:.0}", value)
    }
}
