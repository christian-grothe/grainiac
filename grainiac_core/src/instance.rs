use crate::{
    constants::{BUFFER_SIZE_SECONDS_MAX, BUFFER_SIZE_SECONDS_RECORD, GRAIN_NUM},
    grain::GrainData,
    voice::{self, Voice},
    State, BAR_NUM, VOICE_NUM,
};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Mode {
    Grain,
    Tape,
}

pub struct Instance {
    pub buffer: Vec<f32>,
    pub buffer_to_draw: BufferToDraw,
    pub rec_buffer_size: usize,
    pub max_buffer_size: usize,
    pub current_buffer_size: usize,
    pub write_index: usize,
    pub voices: Vec<Voice>,
    pub grain_data: Vec<GrainData>,
    pub state: State,
}

impl Instance {
    pub fn new(sample_rate: f32) -> Self {
        let max_buffer_size = (BUFFER_SIZE_SECONDS_MAX * sample_rate) as usize;
        let rec_buffer_size = (BUFFER_SIZE_SECONDS_RECORD * sample_rate) as usize;
        let loop_area = (0.25, 0.5);
        Self {
            buffer: vec![0.0; max_buffer_size],
            buffer_to_draw: BufferToDraw::new(BAR_NUM),
            rec_buffer_size,
            max_buffer_size,
            current_buffer_size: rec_buffer_size,
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
        self.buffer_to_draw.resize(self.rec_buffer_size);
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

        if self.write_index >= self.rec_buffer_size {
            self.write_index = 0;
            self.state.is_recording = false;
            self.buffer_to_draw.reset();
        }
    }

    pub fn load_audio(&mut self, samples: Vec<f32>) {
        let sample_num = samples.len();

        if sample_num <= self.max_buffer_size {
            for voice in self.voices.iter_mut() {
                voice.resize(sample_num);
                voice.set_play_speed(self.state.play_speed);
            }

            self.current_buffer_size = sample_num;

            self.buffer_to_draw.resize(sample_num);
            for sample in samples.iter() {
                self.buffer_to_draw.update(*sample);
            }

            self.buffer[..samples.len()].copy_from_slice(&samples);
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
                    let play_index_int = voice.play_pos.floor() as usize;
                    let next_index = (play_index_int + 1) % self.current_buffer_size;
                    let frac = voice.play_pos - play_index_int as f32;

                    let next_sample = self.buffer[play_index_int] * (1.0 - frac as f32)
                        + self.buffer[next_index] * frac as f32;

                    output.0 += next_sample * voice.gain;
                    output.1 += next_sample * voice.gain;
                }
            }
        }

        for grain_data in self.grain_data.iter() {
            let play_index_int = grain_data.pos as usize;
            let next_index = (play_index_int + 1) % self.current_buffer_size;
            let frac = grain_data.pos - play_index_int as f32;

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

pub struct BufferToDraw {
    pub buffer: Vec<f32>,
    samples_per_bar: usize,
    sample_counter: usize,
    current_bar: usize,
    sample_sum: f32,
}

impl BufferToDraw {
    fn new(original_size: usize) -> Self {
        Self {
            buffer: vec![0.0; BAR_NUM],
            samples_per_bar: (original_size as f32 / BAR_NUM as f32) as usize,
            sample_counter: 0,
            current_bar: 0,
            sample_sum: 0.0,
        }
    }

    pub fn update(&mut self, mut sample: f32) {
        let threshold = 0.35;
        let ratio = 3.0;

        if sample < threshold {
            sample *= ratio;
        }

        self.sample_sum += sample * sample;
        self.sample_counter += 1;

        if self.sample_counter >= self.samples_per_bar {
            let mean_square = self.sample_sum / self.samples_per_bar as f32;
            self.buffer[self.current_bar] = mean_square.sqrt();

            // Reset for next bar
            self.sample_sum = 0.0;
            self.sample_counter = 0;
            self.current_bar += 1;
        }
    }

    pub fn reset(&mut self) {
        self.sample_sum = 0.0;
        self.sample_counter = 0;
        self.current_bar = 0;
    }

    pub fn resize(&mut self, vec_size: usize) {
        self.reset();
        self.buffer.fill(0.0);
        self.samples_per_bar = (vec_size as f32 / BAR_NUM as f32) as usize;
    }
}
