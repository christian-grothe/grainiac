use std::{io, time::Duration};

use crossbeam::channel::Sender;
use grainiac_core::{DrawData, Output};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};

use crate::Preset;

pub struct State {
    pub exiting: bool,
    pub out_buf: Output<Vec<DrawData>>,
    pub s: Sender<Preset>,
}

impl State {
    pub fn new(out_buf: Output<Vec<DrawData>>, s: Sender<Preset>) -> Self {
        Self {
            exiting: false,
            out_buf,
            s,
        }
    }

    pub fn handle_event(&mut self, ms: u64, presets: &Vec<Preset>) -> io::Result<()> {
        if event::poll(Duration::from_millis(ms))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    match key_event.code {
                        KeyCode::Esc => self.exiting = true,
                        KeyCode::Char('1') => {
                            if let Some(preset) = presets.get(0) {
                                self.s.send(preset.clone()).unwrap()
                            }
                        }
                        KeyCode::Char('2') => {
                            if let Some(preset) = presets.get(1) {
                                self.s.send(preset.clone()).unwrap()
                            }
                        }
                        KeyCode::Char('3') => {
                            if let Some(preset) = presets.get(2) {
                                self.s.send(preset.clone()).unwrap()
                            }
                        }
                        KeyCode::Char('4') => {
                            if let Some(preset) = presets.get(3) {
                                self.s.send(preset.clone()).unwrap()
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}
