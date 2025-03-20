use std::{io, time::Duration};

use crossbeam::channel::Sender;
use grainiac_core::{DrawData, Output};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};

use crate::{Msg, Preset};

pub enum PresetMode {
    Save,
    Load,
}

pub struct State {
    pub exiting: bool,
    pub out_buf: Output<Vec<DrawData>>,
    pub preset_mode: PresetMode,
    pub presets: Vec<Preset>,
    pub s: Sender<Msg>,
}

impl State {
    pub fn new(out_buf: Output<Vec<DrawData>>, s: Sender<Msg>, presets: Vec<Preset>) -> Self {
        Self {
            exiting: false,
            out_buf,
            preset_mode: PresetMode::Load,
            presets,
            s,
        }
    }

    pub fn handle_event(&mut self, ms: u64) -> io::Result<()> {
        if event::poll(Duration::from_millis(ms))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    match key_event.code {
                        KeyCode::Esc => self.exiting = true,
                        KeyCode::Char('m') => match self.preset_mode {
                            PresetMode::Save => self.preset_mode = PresetMode::Load,
                            PresetMode::Load => self.preset_mode = PresetMode::Save,
                        },
                        KeyCode::Char(c) => match self.preset_mode {
                            PresetMode::Load => {
                                if let Some(preset) = self.presets.iter().find(|p| p.char == c) {
                                    self.s.send(Msg::ApplyPreset(preset.clone())).unwrap();
                                }
                            }
                            PresetMode::Save => self.s.send(Msg::SavePreset(c)).unwrap(),
                        },
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}
