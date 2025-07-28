use std::{cmp::Ordering, ops::AddAssign};

use crate::{
    constants::{BUFFER_SIZE_SECONDS_RECORD, GRAIN_NUM},
    grain::GrainData,
    instance::Mode,
};

use super::grain::Grain;

#[derive(Clone, Debug, Copy)]
pub enum PlayDirection {
    Forward,
    Backward,
}

impl Default for PlayDirection {
    fn default() -> Self {
        PlayDirection::Forward
    }
}

#[derive(Default)]
pub struct PlayHead {
    index: usize,
    fraction: f32,
}

impl AddAssign<f32> for PlayHead {
    fn add_assign(&mut self, rhs: f32) {
        let val_int = rhs.floor();
        let val_frac = rhs - val_int as f32;

        self.index += val_int as usize;
        self.fraction = val_frac;
    }
}

impl PartialEq<usize> for PlayHead {
    fn eq(&self, other: &usize) -> bool {
        self.index == *other
    }
}

impl PartialOrd<usize> for PlayHead {
    fn partial_cmp(&self, other: &usize) -> Option<Ordering> {
        self.index.partial_cmp(other)
    }
}

pub struct Voice {
    pub env: Envelope,
    pub is_playing: bool,
    pub midi_note: usize,
    pub loop_start: f32,
    pub loop_length: f32,
    anti_clip: Envelope,
    grains: [Grain; GRAIN_NUM],
    grain_trigger: Trigger,
    play_dircetion: PlayDirection,
    grain_dircetion: PlayDirection,
    buffersize: usize,
    pub play_pos: f32,
    pub play_head: PlayHead,
    speed: f32,
    sample_rate: f32,
    pitch: f32,
    global_pitch: i8,
    pub gain: f32,
    spray: f32,
    spread: f32,
    pan: f32,
    grain_length: f32,
    grain_data: Vec<GrainData>,
}

impl Voice {
    pub fn new(sample_rate: f32, loop_area: (f32, f32)) -> Self {
        let buffersize = (BUFFER_SIZE_SECONDS_RECORD * sample_rate) as usize;
        let speed = 1.0 / buffersize as f32;
        Self {
            grains: [Grain::default(); GRAIN_NUM],
            grain_trigger: Trigger::new(48000.0, 10.0),
            play_dircetion: PlayDirection::Forward,
            grain_dircetion: PlayDirection::Forward,
            env: Envelope::new(sample_rate),
            anti_clip: Envelope::from(sample_rate, 0.001, 0.001, EnvelopeState::Attack),
            is_playing: false,
            midi_note: 0,
            buffersize,
            play_pos: 0.25,
            play_head: PlayHead::default(),
            loop_start: loop_area.0,
            loop_length: loop_area.1,
            speed,
            sample_rate,
            pitch: 1.0,
            global_pitch: 0,
            gain: 0.0,
            grain_length: 0.25,
            grain_data: vec![GrainData::default(); GRAIN_NUM],
            spread: 1.0,
            spray: 0.0,
            pan: 0.0,
        }
    }

    pub fn resize(&mut self, buffersize: usize) {
        self.buffersize = buffersize;
    }

    pub fn set_play_direction(&mut self, play_direction: PlayDirection) {
        self.play_dircetion = play_direction;
    }

    pub fn set_grain_direction(&mut self, grain_direction: PlayDirection) {
        self.grain_dircetion = grain_direction;
    }

    pub fn set_play_speed(&mut self, speed: f32) {
        self.speed = speed;
    }

    pub fn set_loop_start(&mut self, loop_start: f32) {
        self.loop_start = loop_start;
    }

    pub fn set_loop_length(&mut self, loop_length: f32) {
        self.loop_length = loop_length;
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

    pub fn set_global_pitch(&mut self, global_pitch: i8) {
        self.global_pitch = global_pitch;
    }

    pub fn note_on(&mut self, midi_note: usize) {
        let loop_start_abs = self.loop_start * self.buffersize as f32;
        self.is_playing = true;
        self.midi_note = midi_note;
        self.pitch = 2.0f32.powf((midi_note as f32 - 60.0) / 12.0);
        self.play_pos = loop_start_abs as f32;
        self.env.set_state(EnvelopeState::Attack);
    }

    pub fn note_off(&mut self) {
        self.env.set_state(EnvelopeState::Release);
    }

    pub fn is_release(&self) -> bool {
        self.env.state == EnvelopeState::Release
    }

    pub fn render(&mut self, mode: Mode) -> Vec<GrainData> {
        let loop_start_abs = (self.loop_start * self.buffersize as f32) as f32;
        let loop_end_abs = ((self.loop_start + self.loop_length) * self.buffersize as f32)
            .clamp(0.0, self.buffersize as f32) as f32;

        match self.play_dircetion {
            PlayDirection::Forward => {
                self.play_pos = match mode {
                    Mode::Grain => self.play_pos + self.speed as f32,
                    Mode::Tape => self.play_pos + (1.0 * self.pitch as f32),
                };

                if self.play_pos >= loop_end_abs - 10.0
                    && self.anti_clip.state != EnvelopeState::Release
                {
                    self.anti_clip.state = EnvelopeState::Release;
                }

                if self.play_pos > loop_end_abs || self.play_pos < loop_start_abs {
                    self.play_pos = loop_start_abs;
                    self.anti_clip.state = EnvelopeState::Attack;
                }
            }
            PlayDirection::Backward => {
                self.play_pos = match mode {
                    Mode::Grain => self.play_pos - self.speed as f32,
                    Mode::Tape => self.play_pos - (1.0 * self.pitch) as f32,
                };

                if self.play_pos <= loop_start_abs + 10.0
                    && self.anti_clip.state != EnvelopeState::Release
                {
                    self.anti_clip.state = EnvelopeState::Release;
                }

                if self.play_pos < loop_start_abs {
                    self.play_pos = loop_end_abs;
                    self.anti_clip.state = EnvelopeState::Attack;
                }
            }
        }

        if self.grain_trigger.update() && mode == Mode::Grain {
            for grain in self.grains.iter_mut() {
                let mut pos = self.play_pos
                    + (self.spray * ((fastrand::f32() * self.sample_rate) - self.sample_rate * 0.5))
                        as f32;

                if pos < 0.0 {
                    pos = self.buffersize as f32 + pos;
                } else if pos > self.buffersize as f32 {
                    pos = pos - self.buffersize as f32;
                }

                let main_pitch = 2.0f32.powf(self.global_pitch as f32 / 12.0);

                if !grain.active {
                    let stereo_pos = self.pan + self.spread * ((fastrand::f32() * 2.0) - 1.0);
                    grain.activate(
                        (self.sample_rate * self.grain_length) as usize,
                        pos as f32,
                        self.pitch * main_pitch,
                        self.buffersize,
                        stereo_pos.clamp(-1.0, 1.0),
                        self.grain_dircetion.clone(),
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

        let clip_gain = self.anti_clip.update();
        let env = self.env.update();
        if mode == Mode::Tape {
            self.gain = env * clip_gain;
        } else {
            self.gain = env;
        }

        if self.env.state == EnvelopeState::Off {
            self.midi_note = 0;
            self.is_playing = false;
            self.grain_trigger.reset();
            for grain in self.grains.iter_mut() {
                grain.reset();
            }
        }

        self.grain_data.clone()
    }
}

struct Trigger {
    phase: f32,
    speedrement: f32,
    is_reset: bool,
    sample_rate: f32,
}

impl Trigger {
    fn new(sample_rate: f32, frequency: f32) -> Self {
        Self {
            phase: 0.0,
            speedrement: frequency / sample_rate,
            is_reset: true,
            sample_rate,
        }
    }

    fn update(&mut self) -> bool {
        if self.is_reset {
            self.is_reset = false;
            return true;
        }

        self.phase = self.phase + self.speedrement;
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
        self.speedrement = frequency / self.sample_rate;
    }
}

#[derive(PartialEq)]
pub enum EnvelopeState {
    Attack,
    Release,
    Hold,
    Off,
}

pub struct Envelope {
    inc_attack: f32,
    inc_release: f32,
    gain: f32,
    state: EnvelopeState,
}

impl Envelope {
    fn new(sample_rate: f32) -> Self {
        Self {
            inc_attack: 1.0 / sample_rate,
            inc_release: 1.0 / sample_rate,
            gain: 0.0,
            state: EnvelopeState::Off,
        }
    }

    fn from(sample_rate: f32, attack: f32, release: f32, state: EnvelopeState) -> Self {
        Self {
            inc_attack: 1.0 / (sample_rate * attack),
            inc_release: 1.0 / (sample_rate * release),
            gain: 0.0,
            state,
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
