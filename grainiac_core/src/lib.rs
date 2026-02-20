use rtsan_standalone::nonblocking;
pub use triple_buffer::{triple_buffer, Input, Output};
use voice::PlayDirection;

use crate::constants::BUFFER_SIZE_SECONDS_RECORD;
pub use crate::{
    constants::{BAR_NUM, GRAIN_NUM, VOICE_NUM},
    instance::{Instance, Mode},
};

mod constants;
mod grain;
pub mod instance;
pub mod voice;

#[allow(dead_code)]
#[derive(Clone)]
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
    pub mode: Mode,
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
            mode: Mode::Grain,
        }
    }
}

#[derive(Clone)]
pub struct DrawData {
    pub grain_data: Vec<Option<(f32, f32, f32)>>,
    pub play_heads: Vec<Option<f32>>,
    pub buffer: Vec<f32>,
    pub state: State,
    pub input_peak: f32,
    pub output_peak: f32,
}

impl DrawData {
    pub fn new() -> Self {
        Self {
            grain_data: vec![None; VOICE_NUM * GRAIN_NUM],
            play_heads: vec![None; VOICE_NUM],
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
    l_select: bool,
    r_select: bool,
}

impl Sampler {
    pub fn new(sample_rate: f32, instance_num: usize) -> (Self, Output<Vec<DrawData>>) {
        let (buf_input, buf_output) = triple_buffer(&vec![DrawData::new(); instance_num]);
        (
            Self {
                instances: {
                    let mut instances: Vec<Instance> = Vec::with_capacity(instance_num);
                    for _ in 0..instance_num {
                        instances.push(Instance::new(sample_rate))
                    }
                    instances
                },
                draw_data: buf_input,
                draw_data_update_count: 0,
                sample_rate,
                input_peak: PeakFollower::new(250.0, sample_rate),
                output_peak: PeakFollower::new(250.0, sample_rate),
                l_select: false,
                r_select: true,
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
            let draw_data = self.draw_data.input_buffer_mut();
            for (i, instance) in self.instances.iter().enumerate() {
                draw_data[i].grain_data.fill(None);
                for (index, data) in instance.grain_data.iter().enumerate() {
                    if instance.state.gain == 0.0 {
                        break;
                    }
                    draw_data[i].grain_data[index] = Some((
                        data.pos / instance.current_buffer_size as f32,
                        data.gain,
                        data.stereo_pos,
                    ));
                }

                draw_data[i].play_heads.fill(None);
                if instance.state.mode == Mode::Tape {
                    for (index, voice) in instance.voices.iter().enumerate() {
                        if voice.midi_note != 0 {
                            draw_data[i].play_heads[index] = Some(voice.play_pos);
                        }
                    }
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
        let chunk_size = (BUFFER_SIZE_SECONDS_RECORD * self.sample_rate) as usize;
        let chunks = bufs.chunks(chunk_size);

        for (instance, chunk) in self.instances.iter_mut().zip(chunks) {
            //instance.buffer = chunk.to_vec();
            instance.buffer.copy_from_slice(chunk);

            instance.buffer_to_draw.reset();
            for sample in instance.buffer.iter() {
                instance.buffer_to_draw.update(*sample);
            }
        }
    }

    pub fn load_buf(&mut self, buf: Vec<f32>, index: usize) {
        if let Some(instance) = self.instances.get_mut(index) {
            instance.load_audio(buf);
        }
    }

    #[nonblocking]
    pub fn render(&mut self, stereo_slice: (&mut f32, &mut f32)) {
        let mut output_l = 0.0;
        let mut output_r = 0.0;

        if !self.r_select {
            *stereo_slice.1 = 0.0;
        }

        if !self.l_select {
            *stereo_slice.0 = 0.0;
        }

        let mono = *stereo_slice.0 + *stereo_slice.1;

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

    pub fn set_select_r(&mut self, select: bool) {
        self.r_select = select
    }

    pub fn set_select_l(&mut self, select: bool) {
        self.l_select = select
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

    pub fn toggle_mode(&mut self, index: usize) {
        if let Some(instance) = self.instances.get_mut(index) {
            match instance.state.mode {
                Mode::Tape => instance.set_mode(Mode::Grain),
                Mode::Grain => instance.set_mode(Mode::Tape),
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
