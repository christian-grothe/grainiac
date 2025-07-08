use rtsan_standalone::nonblocking;
pub use triple_buffer::{triple_buffer, Input, Output};
use voice::{PlayDirection, Voice, BUFFER_SIZE_SECONDS, GRAIN_NUM};

mod grain;
pub mod voice;

const VOICE_NUM: usize = 16;
pub const INSTANCE_NUM: usize = 4;
pub const BAR_NUM: usize = 100;
pub const RMS_WINDOW: usize = 1024;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct State {
    pub loop_start: f32,
    pub loop_length: f32,
    pub density: f32,
    pub grain_length: f32,
    pub play_speed: f32,
    pub spray: f32,
    pub pan: f32,
    pub spread: f32,
    pub attack: f32,
    pub release: f32,
    pub pitch: i8,
    pub gain: f32,
    pub is_recording: bool,
    pub is_hold: bool,
    pub play_dir: PlayDirection,
    pub grain_dir: PlayDirection,
}

impl State {
    fn new() -> Self {
        State {
            loop_start: 0.25,
            loop_length: 0.5,
            density: 0.5,
            grain_length: 0.5,
            play_speed: 1.0,
            spray: 0.1,
            pan: 0.0,
            spread: 1.0,
            attack: 0.25,
            release: 0.25,
            pitch: 0,
            gain: 0.5,
            is_recording: false,
            is_hold: false,
            play_dir: PlayDirection::Forward,
            grain_dir: PlayDirection::Forward,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DrawData {
    pub grain_data: Vec<Option<(f32, f32, f32)>>,
    pub buffer: Vec<f32>,
    pub state: State,
    pub input_peak: f32,
    pub output_peak: f32,
}

impl DrawData {
    pub fn new() -> Self {
        Self {
            grain_data: vec![None; VOICE_NUM * GRAIN_NUM],
            buffer: vec![0.0; BAR_NUM],
            state: State::new(),
            input_peak: 0.0,
            output_peak: 0.0,
        }
    }
}

pub struct PeakFollower {
    pub value: f32,
    release_coeff: f32,
}

impl PeakFollower {
    pub fn new(release_time_ms: f32, sample_rate: f32) -> Self {
        let release_samples = (release_time_ms * 0.001) * sample_rate;
        let release_coeff = if release_samples > 0.0 {
            (-1.0 / release_samples).exp()
        } else {
            0.0
        };

        PeakFollower {
            value: 0.0,
            release_coeff,
        }
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        let abs_s = sample.abs();
        if abs_s > self.value {
            self.value = abs_s;
        } else {
            self.value *= self.release_coeff;
        }
        self.value
    }
}

pub struct Sampler {
    instances: Vec<Instance>,
    pub draw_data: Input<Vec<DrawData>>,
    draw_data_update_count: usize,
    sample_rate: f32,
    input_peak: PeakFollower,
    output_peak: PeakFollower,
}

impl Sampler {
    pub fn new(sample_rate: f32) -> (Self, Output<Vec<DrawData>>) {
        let (buf_input, buf_output) = triple_buffer(&vec![DrawData::new(); INSTANCE_NUM]);
        (
            Self {
                instances: {
                    let mut instances: Vec<Instance> = Vec::with_capacity(INSTANCE_NUM);
                    for _ in 0..INSTANCE_NUM {
                        instances.push(Instance::new(sample_rate))
                    }
                    instances
                },
                draw_data: buf_input,
                draw_data_update_count: 0,
                sample_rate,
                input_peak: PeakFollower::new(250.0, sample_rate),
                output_peak: PeakFollower::new(250.0, sample_rate),
            },
            buf_output,
        )
    }

    pub fn record(&mut self, instance_index: usize) {
        if let Some(instance) = self.instances.get_mut(instance_index) {
            instance.record();
        }
    }

    fn get_draw_data(&mut self) {
        self.draw_data_update_count += 1;
        if self.draw_data_update_count >= self.sample_rate as usize / 33 {
            let draw_data = self.draw_data.input_buffer();
            for (i, instance) in self.instances.iter().enumerate() {
                draw_data[i].grain_data.fill(None);
                for (index, data) in instance.grain_data.iter().enumerate() {
                    if instance.state.gain == 0.0 {
                        break;
                    }
                    draw_data[i].grain_data[index] = Some(*data);
                }

                for (index, data) in instance.buffer_to_draw.buffer.iter().enumerate() {
                    draw_data[i].buffer[index] = *data;
                }

                draw_data[i].state = instance.state.clone();
                draw_data[i].input_peak = self.input_peak.value;
                draw_data[i].output_peak = self.output_peak.value;
            }

            self.draw_data.publish();
            self.draw_data_update_count = 0;
        }
    }

    pub fn get_bufs(&mut self) -> Vec<&Vec<f32>> {
        let mut comb = vec![];
        for instance in self.instances.iter() {
            comb.push(&instance.buffer)
        }

        comb
    }

    pub fn load_bufs(&mut self, bufs: Vec<f32>) {
        let chunk_size = (BUFFER_SIZE_SECONDS * self.sample_rate) as usize;
        let chunks = bufs.chunks(chunk_size);

        for (instance, chunk) in self.instances.iter_mut().zip(chunks) {
            instance.buffer = chunk.to_vec();

            instance.buffer_to_draw.reset();
            for sample in instance.buffer.iter() {
                instance.buffer_to_draw.update(*sample);
            }
        }
    }

    #[nonblocking]
    pub fn render(&mut self, stereo_slice: (&mut f32, &mut f32)) {
        let mut output_l = 0.0;
        let mut output_r = 0.0;
        let mono = *stereo_slice.0 + *stereo_slice.1;

        // let threshold = 0.25;
        // let ratio = 4.0;

        // let gain = if mono.abs() < threshold {
        //     1.0 + (ratio - 1.0) * (1.0 - (mono.abs() / threshold))
        // } else {
        //     1.0
        // };

        // mono *= gain;
        // mono = mono.clamp(-1.0, 1.0);

        self.input_peak.process(mono);

        for instance in self.instances.iter_mut() {
            let (l, r) = instance.render(&mono);
            output_l += l;
            output_r += r;
        }

        self.output_peak.process(output_l + output_r);

        self.get_draw_data();

        *stereo_slice.0 = output_l;
        *stereo_slice.1 = output_r;
    }

    pub fn note_on(&mut self, midi_note: usize) {
        for instance in self.instances.iter_mut() {
            if !instance.state.is_hold {
                for voice in instance.voices.iter_mut() {
                    if !voice.is_playing {
                        voice.note_on(midi_note);
                        break;
                    }
                }
            }
        }
    }

    pub fn note_off(&mut self, midi_note: usize) {
        for instance in self.instances.iter_mut() {
            if !instance.state.is_hold {
                for voice in instance.voices.iter_mut() {
                    if voice.midi_note == midi_note && !voice.is_release() {
                        voice.note_off();
                        break;
                    }
                }
            }
        }
    }

    pub fn set_play_dir_from_preset(&mut self, index: usize, value: u8) {
        if let Some(instance) = self.instances.get_mut(index) {
            if value == 0 {
                instance.state.play_dir = PlayDirection::Forward;
            } else {
                instance.state.play_dir = PlayDirection::Backward;
            }
        }
    }

    pub fn set_grain_dir_from_preset(&mut self, index: usize, value: u8) {
        if let Some(instance) = self.instances.get_mut(index) {
            if value == 0 {
                instance.state.grain_dir = PlayDirection::Forward;
            } else {
                instance.state.grain_dir = PlayDirection::Backward;
            }
        }
    }

    pub fn toggle_play_dir(&mut self, index: usize) {
        if let Some(instance) = self.instances.get_mut(index) {
            match instance.state.play_dir {
                PlayDirection::Forward => instance.state.play_dir = PlayDirection::Backward,
                PlayDirection::Backward => instance.state.play_dir = PlayDirection::Forward,
            }
            for voice in instance.voices.iter_mut() {
                voice.set_play_direction(instance.state.play_dir.clone());
            }
        }
    }

    pub fn toggle_grain_dir(&mut self, index: usize) {
        if let Some(instance) = self.instances.get_mut(index) {
            match instance.state.grain_dir {
                PlayDirection::Forward => instance.state.grain_dir = PlayDirection::Backward,
                PlayDirection::Backward => instance.state.grain_dir = PlayDirection::Forward,
            }
            for voice in instance.voices.iter_mut() {
                voice.set_grain_direction(instance.state.grain_dir.clone());
            }
        }
    }

    pub fn set_loop_start(&mut self, index: usize, value: f32) {
        if let Some(instance) = self.instances.get_mut(index) {
            instance.set_loop_start(value);
        }
    }

    pub fn set_loop_length(&mut self, index: usize, value: f32) {
        if let Some(instance) = self.instances.get_mut(index) {
            instance.set_loop_length(value);
        }
    }

    pub fn set_play_speed(&mut self, index: usize, value: f32) {
        if let Some(instance) = self.instances.get_mut(index) {
            instance.set_play_speed(value);
        }
    }

    pub fn set_density(&mut self, index: usize, value: f32) {
        if let Some(instance) = self.instances.get_mut(index) {
            instance.set_density(value);
        }
    }

    pub fn set_spray(&mut self, index: usize, value: f32) {
        if let Some(instance) = self.instances.get_mut(index) {
            instance.set_spray(value);
        }
    }

    pub fn set_grain_length(&mut self, index: usize, value: f32) {
        if let Some(instance) = self.instances.get_mut(index) {
            instance.set_grain_length(value);
        }
    }

    pub fn toggle_hold(&mut self, index: usize) {
        if let Some(instance) = self.instances.get_mut(index) {
            instance.toggle_hold();
        }
    }

    pub fn set_attack(&mut self, index: usize, value: f32) {
        if let Some(instance) = self.instances.get_mut(index) {
            instance.state.attack = value;
            for voice in instance.voices.iter_mut() {
                voice.set_attack(value);
            }
        }
    }

    pub fn set_release(&mut self, index: usize, value: f32) {
        if let Some(instance) = self.instances.get_mut(index) {
            instance.state.release = value;
            for voice in instance.voices.iter_mut() {
                voice.set_release(value);
            }
        }
    }

    pub fn set_global_pitch(&mut self, index: usize, value: i8) {
        if let Some(instance) = self.instances.get_mut(index) {
            instance.state.pitch = value;
            for voice in instance.voices.iter_mut() {
                voice.set_global_pitch(value);
            }
        }
    }

    pub fn set_gain(&mut self, index: usize, value: f32) {
        if let Some(instance) = self.instances.get_mut(index) {
            instance.set_gain(value);
        }
    }

    pub fn set_spread(&mut self, index: usize, value: f32) {
        if let Some(instance) = self.instances.get_mut(index) {
            instance.state.spread = value;
            for voice in instance.voices.iter_mut() {
                voice.set_spread(value);
            }
        }
    }

    pub fn set_pan(&mut self, index: usize, value: f32) {
        if let Some(instance) = self.instances.get_mut(index) {
            instance.state.pan = value;
            for voice in instance.voices.iter_mut() {
                voice.set_pan(value);
            }
        }
    }
}

struct Instance {
    buffer: Vec<f32>,
    buffer_to_draw: BufferToDraw,
    write_index: usize,
    voices: Vec<Voice>,
    grain_data: Vec<(f32, f32, f32)>,
    state: State,
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

    fn set_play_speed(&mut self, value: f32) {
        self.state.play_speed = value;
        for voice in self.voices.iter_mut() {
            voice.set_play_speed(value);
        }
    }

    fn set_loop_start(&mut self, value: f32) {
        self.state.loop_start = value.clamp(0.0, 0.99);
        for voice in self.voices.iter_mut() {
            voice.set_loop_start(value.clamp(0.0, 0.99));
        }
    }

    fn set_loop_length(&mut self, value: f32) {
        self.state.loop_length = value;
        for voice in self.voices.iter_mut() {
            voice.set_loop_length(value);
        }
    }

    fn set_density(&mut self, value: f32) {
        self.state.density = value;
        for voice in self.voices.iter_mut() {
            voice.set_density(value);
        }
    }

    fn set_spray(&mut self, value: f32) {
        self.state.spray = value;
        for voice in self.voices.iter_mut() {
            voice.set_spray(value);
        }
    }

    fn set_grain_length(&mut self, value: f32) {
        self.state.grain_length = value;
        for voice in self.voices.iter_mut() {
            voice.set_grain_length(value);
        }
    }

    fn set_gain(&mut self, value: f32) {
        self.state.gain = value;
    }

    fn toggle_hold(&mut self) {
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

        self.grain_data.clear();
        for voice in self.voices.iter_mut() {
            if voice.midi_note != 0 {
                self.grain_data.extend(voice.render());
            }
        }

        let mut output = (0.0, 0.0);
        for (pos, gain, stereo_pos) in self.grain_data.iter() {
            let play_index_int = (pos * self.buffer.len() as f32) as usize;
            let next_index = (play_index_int + 1) % self.buffer.len();
            let frac = pos * self.buffer.len() as f32 - play_index_int as f32;

            let left_gain = 0.5 * (1.0 - stereo_pos) * self.state.gain;
            let right_gain = 0.5 * (1.0 + stereo_pos) * self.state.gain;

            let next_sample =
                self.buffer[play_index_int] * (1.0 - frac) + self.buffer[next_index] * frac;

            output.0 += next_sample * gain * left_gain;
            output.1 += next_sample * gain * right_gain;
        }

        output.0 *= 0.25;
        output.1 *= 0.25;
        output
    }
}

struct BufferToDraw {
    buffer: Vec<f32>,
    samples_per_bar: usize,
    sample_counter: usize,
    current_bar: usize,
    sample_sum: f32,
}

impl BufferToDraw {
    fn new(bars: usize, original_size: usize) -> Self {
        Self {
            buffer: vec![0.0; bars],
            samples_per_bar: (original_size as f32 / bars as f32) as usize,
            sample_counter: 0,
            current_bar: 0,
            sample_sum: 0.0,
        }
    }

    fn update(&mut self, mut sample: f32) {
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

    fn reset(&mut self) {
        self.sample_sum = 0.0;
        self.sample_counter = 0;
        self.current_bar = 0;
    }
}
