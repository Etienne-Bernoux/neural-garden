// Layout principal du dashboard TUI.
// Decoupe l'ecran en 5 zones : fitness, diversite, cooperation, ile, alertes.

pub mod alerts;
pub mod cooperation;
pub mod diversity;
pub mod fitness;
pub mod island;

use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::Frame;

use crate::snapshot::SimSnapshot;

/// Rendu du dashboard complet.
///
/// Layout :
/// ┌──────────┬──────────┐
/// │ Fitness  │ Diversite│
/// ├──────────┼──────────┤
/// │ Coopera. │ Ile      │
/// ├──────────┴──────────┤
/// │ Alertes             │
/// └─────────────────────┘
pub fn render(frame: &mut Frame, snapshot: &SimSnapshot) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40), // haut
            Constraint::Percentage(40), // milieu
            Constraint::Percentage(20), // bas (alertes)
        ])
        .split(frame.area());

    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_layout[0]);

    let middle = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_layout[1]);

    fitness::render(frame, top[0], snapshot);
    diversity::render(frame, top[1], snapshot);
    cooperation::render(frame, middle[0], snapshot);
    island::render(frame, middle[1], snapshot);
    alerts::render(frame, main_layout[2], snapshot);
}
