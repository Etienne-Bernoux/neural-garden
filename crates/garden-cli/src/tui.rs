// Setup et gestion du terminal crossterm pour le TUI ratatui.

use std::io;

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::snapshot::SimSnapshot;
use crate::ui;

/// Encapsule le terminal ratatui et gere le cycle de vie raw mode.
pub struct Tui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl Tui {
    /// Initialise le terminal en raw mode avec ecran alternatif.
    pub fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    /// Dessine le dashboard a partir d'un snapshot de la simulation.
    pub fn draw(&mut self, snapshot: &SimSnapshot) -> io::Result<()> {
        self.terminal.draw(|frame| {
            ui::render(frame, snapshot);
        })?;
        Ok(())
    }

    /// Restaure le terminal a son etat initial (quitte raw mode).
    pub fn restore(&mut self) -> io::Result<()> {
        disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;
        Ok(())
    }
}

/// Filet de securite : restaure le terminal meme en cas de panic ou d'erreur non geree.
impl Drop for Tui {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
    }
}
