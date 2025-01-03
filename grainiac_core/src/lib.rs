#[cfg(feature = "draw_data")]
use triple_buffer::Input;
use voice::{Voice, BUFFER_SIZE_SECONDS, GRAIN_NUM};

mod grain;
mod voice;

const VOICE_NUM: usize = 16;
pub const INSTANCE_NUM: usize = 4;

#[cfg(feature = "draw_data")]
#[derive(Clone)]
pub struct DrawData {
    pub voice_data: Vec<(f32, f32, f32)>,
    pub loop_area: (f32, f32),
    pub buffer: Vec<f32>,
}

#[cfg(feature = "draw_data")]
impl DrawData {
    pub fn new() -> Self {
        Self {
            voice_data: Vec::with_capacity(VOICE_NUM * GRAIN_NUM),
            loop_area: (0.0, 1.0),
            buffer: vec![0.0; 100],
        }
    }
}

pub struct Sampler {
    instances: Vec<Instance>,
    #[cfg(feature = "draw_data")]
    pub draw_data: Input<Vec<DrawData>>,
    #[cfg(feature = "draw_data")]
    draw_data_update_count: usize,
    #[cfg(feature = "draw_data")]
    sample_rate: f32,
}

impl Sampler {
    #[cfg(feature = "draw_data")]
    pub fn new(sample_rate: f32, buf_input: Input<Vec<DrawData>>) -> Self {
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
        }
    }

    #[cfg(not(feature = "draw_data"))]
    pub fn new(sample_rate: f32) -> Self {
        Self {
            instances: {
                let mut instances: Vec<Instance> = Vec::with_capacity(INSTANCE_NUM);
                for _ in 0..INSTANCE_NUM {
                    instances.push(Instance::new(sample_rate))
                }
                instances
            },
        }
    }

    pub fn record(&mut self, instance_index: usize) {
        if let Some(instance) = self.instances.get_mut(instance_index) {
            instance.record();
        }
    }

    #[cfg(feature = "draw_data")]
    fn get_draw_data(&mut self) {
        self.draw_data_update_count += 1;
        if self.draw_data_update_count >= self.sample_rate as usize / 33 {
            let draw_data = self.draw_data.input_buffer();
            for (i, instance) in self.instances.iter().enumerate() {
                draw_data[i].voice_data.clear();
                draw_data[i].voice_data.extend(instance.voice_data.clone());
                draw_data[i].buffer = instance.buffer_to_draw.buffer.clone();
                draw_data[i].loop_area = instance.loop_area.clone();
            }
            self.draw_data.publish();
            self.draw_data_update_count = 0;
        }
    }

    pub fn render(&mut self, stereo_slice: (&mut f32, &mut f32)) {
        let mut output_l = 0.0;
        let mut output_r = 0.0;
        for instance in self.instances.iter_mut() {
            let (l, r) = instance.render(stereo_slice.0);
            output_l += l;
            output_r += r;
        }
        #[cfg(feature = "draw_data")]
        self.get_draw_data();
        *stereo_slice.0 = output_l;
        *stereo_slice.1 = output_r;
    }

    pub fn note_on(&mut self, midi_note: usize) {
        for instance in self.instances.iter_mut() {
            if !instance.is_hold {
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
            if !instance.is_hold {
                for voice in instance.voices.iter_mut() {
                    if voice.midi_note == midi_note && !voice.is_release() {
                        voice.note_off();
                        break;
                    }
                }
            }
        }
    }

    pub fn set_loop_start(&mut self, index: usize, value: f32) {
        if let Some(instance) = self.instances.get_mut(index) {
            instance.set_loop_start(value);
        }
    }

    #[allow(dead_code)]
    pub fn set_loop_end(&mut self, index: usize, value: f32) {
        if let Some(instance) = self.instances.get_mut(index) {
            instance.set_loop_end(value);
        }
    }

    #[allow(dead_code)]
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
            for voice in instance.voices.iter_mut() {
                voice.set_attack(value);
            }
        }
    }

    pub fn set_release(&mut self, index: usize, value: f32) {
        if let Some(instance) = self.instances.get_mut(index) {
            for voice in instance.voices.iter_mut() {
                voice.set_release(value);
            }
        }
    }

    pub fn set_global_pitch(&mut self, index: usize, value: f32) {
        if let Some(instance) = self.instances.get_mut(index) {
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
            for voice in instance.voices.iter_mut() {
                voice.set_spread(value);
            }
        }
    }

    pub fn set_pan(&mut self, index: usize, value: f32) {
        if let Some(instance) = self.instances.get_mut(index) {
            for voice in instance.voices.iter_mut() {
                voice.set_pan(value);
            }
        }
    }
}

struct Instance {
    buffer: Vec<f32>,
    #[cfg(feature = "draw_data")]
    buffer_to_draw: BufferToDraw,
    write_index: usize,
    is_recording: bool,
    voices: Vec<Voice>,
    voice_data: Vec<(f32, f32, f32)>,
    is_hold: bool,
    loop_area: (f32, f32),
    gain: f32,
}

impl Instance {
    #[cfg(feature = "draw_data")]
    pub fn new(sample_rate: f32) -> Self {
        let buffersize = (BUFFER_SIZE_SECONDS * sample_rate) as usize;
        Self {
            buffer: vec![0.0; buffersize],
            buffer_to_draw: BufferToDraw::new(100, buffersize),
            write_index: 0,
            is_recording: false,
            voices: {
                let mut voices: Vec<Voice> = Vec::with_capacity(VOICE_NUM);
                for _ in 0..VOICE_NUM {
                    voices.push(Voice::new(sample_rate));
                }
                voices
            },
            voice_data: Vec::with_capacity(VOICE_NUM * GRAIN_NUM),
            is_hold: false,
            loop_area: (0.0, 1.0),
            gain: 0.5,
        }
    }
        
    #[cfg(not(feature = "draw_data"))]
    pub fn new(sample_rate: f32) -> Self {
        let buffersize = (BUFFER_SIZE_SECONDS * sample_rate) as usize;
        Self {
            buffer: vec![0.0; buffersize],
            write_index: 0,
            is_recording: false,
            voices: {
                let mut voices: Vec<Voice> = Vec::with_capacity(VOICE_NUM);
                for _ in 0..VOICE_NUM {
                    voices.push(Voice::new(sample_rate));
                }
                voices
            },
            voice_data: Vec::with_capacity(VOICE_NUM * GRAIN_NUM),
            is_hold: false,
            loop_area: (0.0, 1.0),
            gain: 0.5,
        }
    }

    pub fn record(&mut self) {
        self.is_recording = true;
        self.write_index = 0;
        #[cfg(feature = "draw_data")]
        self.buffer_to_draw.reset();
    }

    fn set_play_speed(&mut self, value: f32) {
        for voice in self.voices.iter_mut() {
            voice.set_play_speed(value);
        }
    }

    fn set_loop_start(&mut self, value: f32) {
        self.loop_area.0 = value;
        for voice in self.voices.iter_mut() {
            voice.set_loop_start(value);
        }
    }

    #[allow(dead_code)]
    fn set_loop_end(&mut self, value: f32) {
        self.loop_area.1 = value;
        for voice in self.voices.iter_mut() {
            voice.set_loop_end(value);
        }
    }

    #[allow(dead_code)]
    fn set_loop_length(&mut self, value: f32) {
        self.loop_area.1 = value;
        let mut end = self.loop_area.0 + value;

        if end > 1.0 {
            end = 1.0;
        }

        for voice in self.voices.iter_mut() {
            voice.set_loop_end(end);
        }
    }

    fn set_density(&mut self, value: f32) {
        for voice in self.voices.iter_mut() {
            voice.set_density(value);
        }
    }

    fn set_spray(&mut self, value: f32) {
        for voice in self.voices.iter_mut() {
            voice.set_spray(value);
        }
    }

    fn set_grain_length(&mut self, value: f32) {
        for voice in self.voices.iter_mut() {
            voice.set_grain_length(value);
        }
    }

    fn set_gain(&mut self, value: f32) {
        self.gain = value;
    }

    fn toggle_hold(&mut self) {
        match self.is_hold {
            true => {
                for voice in self.voices.iter_mut() {
                    if voice.midi_note != 0 {
                        voice.env.set_state(voice::EnvelopeState::Release);
                    }
                }
                self.is_hold = false;
            }
            false => {
                for voice in self.voices.iter_mut() {
                    if voice.midi_note != 0 {
                        voice.env.set_state(voice::EnvelopeState::Hold);
                    }
                }
                self.is_hold = true;
            }
        }
    }

    fn write(&mut self, sample: f32) {
        self.buffer[self.write_index] = sample;
        self.write_index = self.write_index + 1;

        #[cfg(feature = "draw_data")]
        self.buffer_to_draw.update(sample);

        if self.write_index >= self.buffer.len() {
            self.write_index = 0;
            self.is_recording = false;
            #[cfg(feature = "draw_data")]
            self.buffer_to_draw.reset();
        }
    }

    pub fn render(&mut self, input_sample: &f32) -> (f32, f32) {
        if self.is_recording {
            self.write(*input_sample);
        }

        self.voice_data.clear();
        for voice in self.voices.iter_mut() {
            if voice.midi_note != 0 {
                self.voice_data.extend(voice.render());
            }
        }

        let mut output = (0.0, 0.0);
        for (pos, gain, stereo_pos) in self.voice_data.iter() {
            let play_index_int = (pos * self.buffer.len() as f32) as usize;
            let next_index = (play_index_int + 1) % self.buffer.len();
            let frac = pos * self.buffer.len() as f32 - play_index_int as f32;

            let left_gain = 0.5 * (1.0 - stereo_pos) * self.gain;
            let right_gain = 0.5 * (1.0 + stereo_pos) * self.gain;

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

#[cfg(feature = "draw_data")]
struct BufferToDraw {
    buffer: Vec<f32>,
    samples_per_bar: usize,
    sample_counter: usize,
    current_bar: usize,
    sample_sum: f32,
}

#[cfg(feature = "draw_data")]
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

    fn update(&mut self, sample: f32) {
        self.sample_sum += sample.abs();
        self.sample_counter += 1;
        if self.sample_counter >= self.samples_per_bar {
            self.buffer[self.current_bar] = self.sample_sum / self.samples_per_bar as f32;
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
