use std::{
    env,
    fs::{self, File},
    io::{self, BufReader},
    time::Duration,
};

use crossbeam::channel::Sender;
use grainiac_core::{DrawData, Output};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};

use crate::{Config, Msg, Preset};

#[derive(PartialEq)]
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
                        KeyCode::Char('x') => {
                            self.s.send(Msg::SaveAudio).unwrap();
                        }
                        KeyCode::Char('y') => {
                            self.s.send(Msg::LoadAudio).unwrap();
                        }
                        KeyCode::Char(c) => match self.preset_mode {
                            PresetMode::Load => {
                                if let Some(preset) = self.presets.iter().find(|p| p.char == c) {
                                    self.s.send(Msg::ApplyPreset(preset.clone())).unwrap();
                                }
                            }
                            PresetMode::Save => self.save_preset(c),
                        },
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        Ok(())
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

        #[allow(deprecated)]
        let home_dir = env::home_dir().unwrap();
        let config_file_path = home_dir.join(".config/grainiac/config.json");

        let file = File::open(config_file_path.clone()).unwrap();
        let reader = BufReader::new(file);

        let mut config: Config = serde_json::from_reader(reader).expect("could not open json");

        config.presets = self.presets.clone();

        let json_string =
            serde_json::to_string(&config).expect("could not transform preset to string");

        fs::write(config_file_path, json_string).expect("Unable to write file");

        self.preset_mode = PresetMode::Load;
    }
}
