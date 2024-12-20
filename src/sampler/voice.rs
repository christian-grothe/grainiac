use super::grain::Grain;

pub const GRAIN_NUM: usize = 40;
pub const BUFFER_SIZE_SECONDS: f32 = 5.0;

#[allow(dead_code)]
enum PlayDirection {
    Forward,
    Backward,
}

#[allow(dead_code)]
pub struct Voice {
    grains: Vec<Grain>,
    grain_trigger: Trigger,
    play_dircetion: PlayDirection,
    env: Envelope,
    pub is_playing: bool,
    pub midi_note: usize,
    buffersize: usize,
    play_pos: f32,
    loop_start: f32,
    loop_end: f32,
    inc: f32,
    sample_rate: f32,
    pitch: f32,
    gain: f32,
    spray: f32,
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
            gain: 0.0,
            spray: 0.1,
            grain_length: 1.0,
            grain_data: Vec::with_capacity(GRAIN_NUM),
        }
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

    pub fn set_grain_length(&mut self, grain_length: f32) {
        self.grain_length = grain_length;
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
                    grain.activate(
                        (self.sample_rate * self.grain_length) as usize,
                        pos,
                        self.pitch,
                        self.buffersize,
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
    }

    fn set_freq(&mut self, frequency: f32) {
        self.increment = frequency / self.sample_rate;
    }
}

#[derive(PartialEq)]
enum EnvelopeState {
    Attack,
    Release,
    Hold,
    Off,
}

#[allow(dead_code)]
struct Envelope {
    inc: f32,
    gain: f32,
    state: EnvelopeState,
    sample_rate: f32,
}

impl Envelope {
    fn new(sample_rate: f32) -> Self {
        Self {
            inc: 1.0 / (sample_rate * 5.0),
            gain: 0.0,
            state: EnvelopeState::Off,
            sample_rate,
        }
    }

    fn update(&mut self) -> f32 {
        match self.state {
            EnvelopeState::Attack => {
                self.gain += self.inc;
                if self.gain >= 1.0 {
                    self.gain = 1.0;
                    self.state = EnvelopeState::Hold;
                }
                self.gain
            }
            EnvelopeState::Release => {
                self.gain -= self.inc;
                if self.gain <= 0.00011 {
                    self.gain = 0.00011;
                    self.state = EnvelopeState::Off;
                }
                self.gain
            }
            _ => self.gain,
        }
    }

    fn set_state(&mut self, state: EnvelopeState) {
        self.state = state;
    }
}
