use anyhow::Result;
use crossterm::event::{self, Event};
use ratatui::{backend::Backend, Terminal};
use std::time::{Duration, Instant};

use crate::commands::handle_key_event;
use crate::state::App;

pub fn draw<B: Backend>(terminal: &mut Terminal<B>, app: &App) -> Result<()> {
    terminal.draw(|f| {
        crate::ui::render_app(f, app);
    })?;
    Ok(())
}

impl App {
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        let mut last_tick = Instant::now();
        let tick_rate = Duration::from_millis(250);

        // Initial draw
        self.refresh_file_list()?;
        draw(terminal, self)?;

        loop {
            // Calculate timeout for event polling
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    // Handle key event
                    if handle_key_event(self, key)? {
                        // Request quit
                        return Ok(());
                    }
                }
            }

            // Redraw if needed
            if last_tick.elapsed() >= tick_rate {
                draw(terminal, self)?;
                last_tick = Instant::now();
            } else {
                draw(terminal, self)?;
            }
        }
    }
}
