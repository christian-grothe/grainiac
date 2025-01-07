use super::grain::Grain;

pub const GRAIN_NUM: usize = 256;
pub const BUFFER_SIZE_SECONDS: f32 = 5.0;

#[allow(dead_code)]
#[derive(Clone)]
pub enum PlayDirection {
    Forward,
    Backward,
}

pub struct Voice {
    pub env: Envelope,
    pub is_playing: bool,
    pub midi_note: usize,
    pub loop_start: f32,
    pub loop_end: f32,
    grains: Vec<Grain>,
    grain_trigger: Trigger,
    play_dircetion: PlayDirection,
    buffersize: usize,
    play_pos: f32,
    inc: f32,
    sample_rate: f32,
    pitch: f32,
    global_pitch: f32,
    gain: f32,
    spray: f32,
    spread: f32,
    pan: f32,
    grain_length: f32,
    grain_data: Vec<(f32, f32, f32)>,
}

impl Voice {
    pub fn new(sample_rate: f32) -> Self {
        let buffersize = (BUFFER_SIZE_SECONDS * sample_rate) as usize;
        let inc = 0.25 / buffersize as f32;
        Self {
            grains: {
                let mut grains: Vec<Grain> = Vec::with_capacity(GRAIN_NUM);
                for _ in 0..GRAIN_NUM {
                    grains.push(Grain::default());
                }
                grains
            },
            grain_trigger: Trigger::new(48000.0, 15.0),
            play_dircetion: PlayDirection::Forward,
            env: Envelope::new(sample_rate),
            is_playing: false,
            midi_note: 0,
            buffersize,
            play_pos: 0.25,
            loop_start: 0.25,
            loop_end: 0.75,
            inc,
            sample_rate,
            pitch: 1.0,
            global_pitch: 1.0,
            gain: 0.0,
            spray: 0.0,
            grain_length: 0.2,
            grain_data: Vec::with_capacity(GRAIN_NUM),
            spread: 0.5,
            pan: 0.0,
        }
    }

    pub fn set_play_direction(&mut self, play_direction: PlayDirection) {
        self.play_dircetion = play_direction;
    }

    pub fn set_play_speed(&mut self, speed: f32) {
        self.inc = speed / self.buffersize as f32;
    }

    pub fn set_loop_start(&mut self, loop_start: f32) {
        self.loop_start = loop_start;
    }

    pub fn set_loop_end(&mut self, loop_end: f32) {
        self.loop_end = loop_end;
    }

    pub fn set_density(&mut self, density: f32) {
        self.grain_trigger.set_freq(density);
    }

    pub fn set_spray(&mut self, spray: f32) {
        self.spray = spray;
    }

    pub fn set_spread(&mut self, spread: f32) {
        self.spread = spread;
    }

    pub fn set_pan(&mut self, pan: f32) {
        self.pan = pan;
    }

    pub fn set_grain_length(&mut self, grain_length: f32) {
        self.grain_length = grain_length;
    }

    pub fn set_attack(&mut self, attack: f32) {
        self.env.inc_attack = 1.0 / (self.sample_rate * attack);
    }

    pub fn set_release(&mut self, release: f32) {
        self.env.inc_release = 1.0 / (self.sample_rate * release);
    }

    pub fn set_global_pitch(&mut self, global_pitch: f32) {
        self.global_pitch = global_pitch;
    }

    pub fn note_on(&mut self, midi_note: usize) {
        self.is_playing = true;
        self.midi_note = midi_note;
        self.pitch = 2.0f32.powf((midi_note as f32 - 60.0) / 12.0);
        self.play_pos = self.loop_start;
        self.env.set_state(EnvelopeState::Attack);
    }

    pub fn note_off(&mut self) {
        self.env.set_state(EnvelopeState::Release);
    }

    pub fn is_release(&self) -> bool {
        self.env.state == EnvelopeState::Release
    }

    pub fn render(&mut self) -> &Vec<(f32, f32, f32)> {
        match self.play_dircetion {
            PlayDirection::Forward => {
                self.play_pos += self.inc;
                if self.play_pos > self.loop_end {
                    self.play_pos = self.loop_start;
                }
            }
            PlayDirection::Backward => {
                self.play_pos -= self.inc;
                if self.play_pos < self.loop_start || self.play_pos <= 0.0 {
                    self.play_pos = self.loop_end;
                }
            }
        }

        if self.grain_trigger.update() {
            for grain in self.grains.iter_mut() {
                let mut pos = self.play_pos + self.spray * ((rand::random::<f32>() * 0.5) - 0.25);

                if pos < 0.0 {
                    pos = 1.0 + pos;
                } else if pos > 1.0 {
                    pos = pos - 1.0;
                }

                if !grain.active {
                    let stereo_pos = self.pan + self.spread * ((rand::random::<f32>() * 2.0) - 1.0);
                    grain.activate(
                        (self.sample_rate * self.grain_length) as usize,
                        pos,
                        self.pitch * self.global_pitch,
                        self.buffersize,
                        stereo_pos.clamp(-1.0, 1.0),
                    );
                    break;
                }
            }
        }

        self.grain_data.clear();
        for grain in self.grains.iter_mut() {
            if grain.active {
                self.grain_data.push(grain.update(self.gain));
            }
        }

        self.gain = self.env.update();

        if self.env.state == EnvelopeState::Off {
            self.midi_note = 0;
            self.is_playing = false;
            self.grain_trigger.reset();
            for grain in self.grains.iter_mut() {
                grain.reset();
            }
        }

        &self.grain_data
    }
}

struct Trigger {
    phase: f32,
    increment: f32,
    is_reset: bool,
    sample_rate: f32,
}

impl Trigger {
    fn new(sample_rate: f32, frequency: f32) -> Self {
        Self {
            phase: 0.0,
            increment: frequency / sample_rate,
            is_reset: true,
            sample_rate,
        }
    }

    fn update(&mut self) -> bool {
        if self.is_reset {
            self.is_reset = false;
            return true;
        }

        self.phase = self.phase + self.increment;
        if self.phase >= 1.0 {
            self.phase = 0.0;
            return true;
        }
        return false;
    }

    fn reset(&mut self) {
        self.phase = 0.0;
        self.is_reset = true;
    }

    fn set_freq(&mut self, frequency: f32) {
        self.increment = frequency / self.sample_rate;
    }
}

#[derive(PartialEq)]
pub enum EnvelopeState {
    Attack,
    Release,
    Hold,
    Off,
}

#[allow(dead_code)]
pub struct Envelope {
    inc_attack: f32,
    inc_release: f32,
    gain: f32,
    state: EnvelopeState,
    sample_rate: f32,
}

impl Envelope {
    fn new(sample_rate: f32) -> Self {
        Self {
            inc_attack: 1.0 / (sample_rate),
            inc_release: 1.0 / (sample_rate),
            gain: 0.0,
            state: EnvelopeState::Off,
            sample_rate,
        }
    }

    fn update(&mut self) -> f32 {
        match self.state {
            EnvelopeState::Attack => {
                self.gain += self.inc_attack;
                if self.gain >= 1.0 {
                    self.gain = 1.0;
                    self.state = EnvelopeState::Hold;
                }
                self.gain
            }
            EnvelopeState::Release => {
                self.gain -= self.inc_release;
                if self.gain <= 0.000011 {
                    self.gain = 0.0;
                    self.state = EnvelopeState::Off;
                }
                self.gain
            }
            _ => self.gain,
        }
    }

    pub fn set_state(&mut self, state: EnvelopeState) {
        self.state = state;
    }
}
