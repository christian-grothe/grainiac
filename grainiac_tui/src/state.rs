use std::{
    env,
    fs::{self, File},
    io::{self, BufReader},
    time::Duration,
};

use crossbeam::channel::Sender;
use grainiac_core::{DrawData, Output};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};

use crate::{Config, Msg, Preset};

#[derive(PartialEq)]
pub enum NumMode {
    SavePreset,
    LoadPreset,
    SaveAudio,
    LoadAudio,
}

impl NumMode {
    fn next(&mut self) {
        *self = match self {
            NumMode::LoadPreset => NumMode::SavePreset,
            NumMode::SavePreset => NumMode::LoadAudio,
            NumMode::LoadAudio => NumMode::SaveAudio,
            NumMode::SaveAudio => NumMode::LoadPreset,
        }
    }
}

#[derive(PartialEq)]
pub enum View {
    Main,
    Preset,
    Audio,
}

impl View {
    fn next(&mut self) {
        *self = match self {
            View::Main => View::Preset,
            View::Preset => View::Audio,
            View::Audio => View::Main,
        }
    }
}

#[allow(dead_code)]
pub struct State {
    pub exiting: bool,
    pub view: View,
    pub out_buf: Output<Vec<DrawData>>,
    pub num_mode: NumMode,
    pub presets: Vec<Preset>,
    pub selected_preset_idx: usize,
    pub selectes_audio_idx: usize,
    pub s: Sender<Msg>,
}

impl State {
    pub fn new(out_buf: Output<Vec<DrawData>>, s: Sender<Msg>, presets: Vec<Preset>) -> Self {
        Self {
            exiting: false,
            view: View::Main,
            out_buf,
            num_mode: NumMode::LoadPreset,
            selected_preset_idx: 0,
            selectes_audio_idx: 0,
            presets,
            s,
        }
    }

    pub fn handle_event(&mut self, ms: u64) -> io::Result<()> {
        if event::poll(Duration::from_millis(ms))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => match self.view {
                    View::Main => self.handle_main_view(key_event),
                    View::Preset => self.handle_preset_view(key_event),
                    View::Audio => self.handle_audio_view(key_event),
                },
                _ => {}
            }
        }
        Ok(())
    }

    fn handle_main_view(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => self.exiting = true,
            KeyCode::Char('m') => self.num_mode.next(),
            KeyCode::Char('n') => self.view.next(),
            KeyCode::Char(c) => {
                if c.is_ascii_digit() {
                    match self.num_mode {
                        NumMode::LoadPreset => {
                            if let Some(preset) = self.presets.iter().find(|p| p.char == c) {
                                self.s.send(Msg::ApplyPreset(preset.clone())).unwrap();
                                self.selected_preset_idx = c.to_digit(10).unwrap() as usize - 1;
                            }
                        }
                        NumMode::SavePreset => self.save_preset(c),
                        NumMode::LoadAudio => self.s.send(Msg::LoadAudio(c)).unwrap(),
                        NumMode::SaveAudio => self.s.send(Msg::SaveAudio(c)).unwrap(),
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_preset_view(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => self.exiting = true,
            KeyCode::Char('n') => self.view.next(),
            KeyCode::Char('j') => {
                self.selected_preset_idx = (self.selected_preset_idx + 1) % self.presets.len()
            }
            KeyCode::Char('k') => {
                self.selected_preset_idx = if self.selected_preset_idx == 0 {
                    self.presets.len() - 1
                } else {
                    self.selected_preset_idx - 1
                }
            }
            KeyCode::Enter => {
                if let Some(preset) = self.presets.iter().find(|p| {
                    (p.char.to_digit(10).unwrap()) as usize == self.selected_preset_idx + 1
                }) {
                    self.s.send(Msg::ApplyPreset(preset.clone())).unwrap();
                    self.view = View::Main;
                }
            }
            _ => {}
        }
    }
    fn handle_audio_view(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => self.exiting = true,
            KeyCode::Char('n') => self.view.next(),
            _ => {}
        }
    }

    fn save_preset(&mut self, char: char) {
        let data = self.out_buf.read();
        let mut new_preset = Preset::default();
        for (i, track) in data.iter().enumerate() {
            new_preset.gain[i] = track.state.gain;
            new_preset.loop_start[i] = track.state.loop_start;
            new_preset.loop_length[i] = track.state.loop_length;
            new_preset.density[i] = track.state.density;
            new_preset.grain_length[i] = track.state.grain_length;
            new_preset.play_speed[i] = track.state.play_speed;
            new_preset.spray[i] = track.state.spray;
            new_preset.pan[i] = track.state.pan;
            new_preset.spread[i] = track.state.spread;
            new_preset.attack[i] = track.state.attack;
            new_preset.release[i] = track.state.release;
            new_preset.pitch[i] = track.state.pitch;
            new_preset.play_dir[i] = match track.state.play_dir {
                grainiac_core::voice::PlayDirection::Forward => 0,
                grainiac_core::voice::PlayDirection::Backward => 1,
            };
            new_preset.grain_dir[i] = match track.state.grain_dir {
                grainiac_core::voice::PlayDirection::Forward => 0,
                grainiac_core::voice::PlayDirection::Backward => 1,
            };
            new_preset.name = format!("preset_{}", char);
            new_preset.char = char;
        }

        if let Some(index) = self.presets.iter().position(|p| p.char == char) {
            self.presets[index] = new_preset;
        } else {
            self.presets.push(new_preset);
        }

        let home_dir = env::home_dir().unwrap();
        let config_file_path = home_dir.join(".config/grainiac/config.json");

        let file = File::open(config_file_path.clone()).unwrap();
        let reader = BufReader::new(file);

        let mut config: Config = serde_json::from_reader(reader).expect("could not open json");

        config.presets = self.presets.clone();

        let json_string =
            serde_json::to_string(&config).expect("could not transform preset to string");

        fs::write(config_file_path, json_string).expect("Unable to write file");

        self.num_mode = NumMode::LoadPreset;
    }
}
