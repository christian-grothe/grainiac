use crate::{
    constants::{BUFFER_SIZE_SECONDS, GRAIN_NUM},
    grain::GrainData,
    voice::{self, Voice},
    BufferToDraw, State, BAR_NUM, VOICE_NUM,
};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Mode {
    Grain,
    Tape,
}

pub struct Instance {
    pub buffer: Vec<f32>,
    pub buffer_to_draw: BufferToDraw,
    pub write_index: usize,
    pub voices: Vec<Voice>,
    pub grain_data: Vec<GrainData>,
    pub state: State,
}

impl Instance {
    pub fn new(sample_rate: f32) -> Self {
        let buffersize = (BUFFER_SIZE_SECONDS * sample_rate) as usize;
        let loop_area = (0.25, 0.5);
        Self {
            buffer: vec![0.0; buffersize],
            buffer_to_draw: BufferToDraw::new(BAR_NUM, buffersize),
            write_index: 0,
            voices: {
                let mut voices: Vec<Voice> = Vec::with_capacity(VOICE_NUM);
                for _ in 0..VOICE_NUM {
                    voices.push(Voice::new(sample_rate, loop_area.clone()));
                }
                voices
            },
            grain_data: Vec::with_capacity(VOICE_NUM * GRAIN_NUM),
            state: State::new(),
        }
    }

    pub fn record(&mut self) {
        self.state.is_recording = true;
        self.write_index = 0;
        self.buffer_to_draw.reset();
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.state.mode = mode;
    }

    pub fn set_play_speed(&mut self, value: f32) {
        self.state.play_speed = value;
        for voice in self.voices.iter_mut() {
            voice.set_play_speed(value);
        }
    }

    pub fn set_loop_start(&mut self, value: f32) {
        self.state.loop_start = value.clamp(0.0, 0.99);
        for voice in self.voices.iter_mut() {
            voice.set_loop_start(value.clamp(0.0, 0.99));
        }
    }

    pub fn set_loop_length(&mut self, value: f32) {
        self.state.loop_length = value;
        for voice in self.voices.iter_mut() {
            voice.set_loop_length(value);
        }
    }

    pub fn set_density(&mut self, value: f32) {
        self.state.density = value;
        for voice in self.voices.iter_mut() {
            voice.set_density(value);
        }
    }

    pub fn set_spray(&mut self, value: f32) {
        self.state.spray = value;
        for voice in self.voices.iter_mut() {
            voice.set_spray(value);
        }
    }

    pub fn set_grain_length(&mut self, value: f32) {
        self.state.grain_length = value;
        for voice in self.voices.iter_mut() {
            voice.set_grain_length(value);
        }
    }

    pub fn set_gain(&mut self, value: f32) {
        self.state.gain = value;
    }

    pub fn toggle_hold(&mut self) {
        match self.state.is_hold {
            true => {
                for voice in self.voices.iter_mut() {
                    if voice.midi_note != 0 {
                        voice.env.set_state(voice::EnvelopeState::Release);
                    }
                }
                self.state.is_hold = false;
            }
            false => {
                for voice in self.voices.iter_mut() {
                    if voice.midi_note != 0 {
                        voice.env.set_state(voice::EnvelopeState::Hold);
                    }
                }
                self.state.is_hold = true;
            }
        }
    }

    fn write(&mut self, sample: f32) {
        self.buffer[self.write_index] = sample;
        self.write_index = self.write_index + 1;

        self.buffer_to_draw.update(sample);

        if self.write_index >= self.buffer.len() {
            self.write_index = 0;
            self.state.is_recording = false;
            self.buffer_to_draw.reset();
        }
    }

    pub fn render(&mut self, input_sample: &f32) -> (f32, f32) {
        if self.state.is_recording {
            self.write(*input_sample);
        }

        let mut output = (0.0, 0.0);

        self.grain_data.clear();
        for voice in self.voices.iter_mut() {
            if voice.midi_note != 0 {
                self.grain_data.extend(voice.render(self.state.mode));

                if self.state.mode == Mode::Tape {
                    let play_index_int = (voice.play_pos * self.buffer.len() as f32) as usize;
                    let next_index = (play_index_int + 1) % self.buffer.len();
                    let frac = voice.play_pos * self.buffer.len() as f32 - play_index_int as f32;

                    let next_sample =
                        self.buffer[play_index_int] * (1.0 - frac) + self.buffer[next_index] * frac;

                    output.0 += next_sample * voice.gain;
                    output.1 += next_sample * voice.gain;
                }
            }
        }

        for grain_data in self.grain_data.iter() {
            let play_index_int = (grain_data.pos * self.buffer.len() as f32) as usize;
            let next_index = (play_index_int + 1) % self.buffer.len();
            let frac = grain_data.pos * self.buffer.len() as f32 - play_index_int as f32;

            let left_gain = 0.5 * (1.0 - grain_data.stereo_pos);
            let right_gain = 0.5 * (1.0 + grain_data.stereo_pos);

            let next_sample =
                self.buffer[play_index_int] * (1.0 - frac) + self.buffer[next_index] * frac;

            output.0 += next_sample * grain_data.gain * left_gain;
            output.1 += next_sample * grain_data.gain * right_gain;
        }

        output.0 *= 0.5 * self.state.gain;
        output.1 *= 0.5 * self.state.gain;

        output
    }
}
