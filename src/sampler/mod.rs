use voice::{Voice, BUFFER_SIZE_SECONDS};

mod grain;
mod voice;

const VOICE_NUM: usize = 16;
const INSTANCE_NUM: usize = 1;

pub struct Sampler {
    instances: Vec<Instance>,
}

impl Sampler {
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

    pub fn render(&mut self, stereo_slice: (&mut f32, &mut f32)) {
        let mut output_l = 0.0;
        let mut output_r = 0.0;
        for instance in self.instances.iter_mut() {
            let (l, r) = instance.render(stereo_slice.0);
            output_l += l;
            output_r += r;
        }
        *stereo_slice.0 = output_l;
        *stereo_slice.1 = output_r;
    }

    pub fn note_on(&mut self, midi_note: usize) {
        for instance in self.instances.iter_mut() {
            for voice in instance.voices.iter_mut() {
                if !voice.is_playing {
                    voice.note_on(midi_note);
                    break;
                }
            }
        }
    }

    pub fn note_off(&mut self, midi_note: usize) {
        for instance in self.instances.iter_mut() {
            for voice in instance.voices.iter_mut() {
                if voice.midi_note == midi_note && !voice.is_release() {
                    voice.note_off();
                    break;
                }
            }
        }
    }

    pub fn set_loop_start(&mut self, index: usize, value: f32) {
        if let Some(instance) = self.instances.get_mut(index) {
            instance.set_loop_start(value);
        }
    }

    pub fn set_loop_end(&mut self, index: usize, value: f32) {
        if let Some(instance) = self.instances.get_mut(index) {
            instance.set_loop_end(value);
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
}

struct Instance {
    buffer: Vec<f32>,
    write_index: usize,
    is_recording: bool,
    voices: Vec<Voice>,
}

impl Instance {
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
        }
    }

    pub fn record(&mut self) {
        nih_plug::nih_log!("recording start");
        self.is_recording = true;
    }

    fn set_play_speed(&mut self, value: f32) {
        for voice in self.voices.iter_mut() {
            voice.set_play_speed(value);
        }
    }

    fn set_loop_start(&mut self, value: f32) {
        for voice in self.voices.iter_mut() {
            voice.set_loop_start(value);
        }
    }

    fn set_loop_end(&mut self, value: f32) {
        for voice in self.voices.iter_mut() {
            voice.set_loop_end(value);
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

    fn write(&mut self, sample: f32) {
        self.buffer[self.write_index] = sample;
        self.write_index = self.write_index + 1;
        if self.write_index >= self.buffer.len() {
            self.write_index = 0;
            self.is_recording = false;
            nih_plug::nih_log!("recording finished");
        }
    }

    pub fn render(&mut self, input_sample: &f32) -> (f32, f32) {
        if self.is_recording {
            self.write(*input_sample);
        }

        let mut voice_data: Vec<(f32, f32, f32)> = vec![];
        for voice in self.voices.iter_mut() {
            if voice.midi_note != 0 {
                voice_data.extend(voice.render());
            }
        }

        nih_plug::nih_log!("grain_num: {:?}", voice_data.len());

        let mut output = (0.0, 0.0);
        for (pos, gain, stereo_pos) in voice_data.iter() {
            let play_index_int = (pos * self.buffer.len() as f32) as usize;
            let next_index = (play_index_int + 1) % self.buffer.len();
            let frac = pos * self.buffer.len() as f32 - play_index_int as f32;

            let left_gain = 0.5 * (1.0 - stereo_pos);
            let right_gain = 0.5 * (1.0 + stereo_pos);

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
