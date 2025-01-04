use std::{io, time::Duration};

use grainiac_core::{DrawData, Output};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};

pub struct State {
    pub exiting: bool,
    pub out_buf: Output<Vec<DrawData>>,
}

impl State {
    pub fn new(out_buf: Output<Vec<DrawData>>) -> Self {
        Self {
            exiting: false,
            out_buf,
        }
    }

    pub fn handle_event(&mut self, ms: u64) -> io::Result<()> {
        if event::poll(Duration::from_millis(ms))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    match key_event.code {
                        KeyCode::Esc => self.exiting = true,
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}
