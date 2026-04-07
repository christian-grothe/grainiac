use std::f64::consts::PI;

use crate::voice::PlayDirection;

#[derive(Default, Clone, Copy)]
pub struct GrainData {
    pub pos: f32,
    pub gain: f32,
    pub stereo_pos: f32,
}

#[derive(Default, Copy, Clone)]
pub struct Grain {
    env: Envelope,
    pub active: bool,
    length: usize,
    counter: usize,
    pos: f32,
    pitch: f32,
    buffersize: usize,
    gain: f32,
    stereo_pos: f32,
    grain_direction: PlayDirection,
    back_and_forth_forward: bool,
}

impl Grain {
    pub fn activate(
        &mut self,
        length: usize,
        start_pos: f32,
        pitch: f32,
        buffersize: usize,
        stereo_pos: f32,
        grain_direction: PlayDirection,
    ) {
        self.active = true;
        self.pos = start_pos;
        self.length = length;
        self.env.set_inc(1.0 / length as f32);
        self.pitch = pitch;
        self.buffersize = buffersize;
        self.stereo_pos = stereo_pos;
        self.grain_direction = grain_direction;
        self.back_and_forth_forward = true;
    }

    pub fn update(&mut self, gain: f32) -> GrainData {
        match self.grain_direction {
            PlayDirection::Forward => {
                self.pos += self.pitch;

                if self.pos >= self.buffersize as f32 {
                    self.pos = 0.0;
                }
            }
            PlayDirection::Backward => {
                self.pos -= self.pitch;

                if self.pos <= 0.0 {
                    self.pos = self.buffersize as f32;
                }
            }
            PlayDirection::BackAndForth => {
                let max_pos = self.buffersize.saturating_sub(1) as f32;
                if self.back_and_forth_forward {
                    self.pos += self.pitch;
                    if self.pos >= max_pos {
                        self.pos = max_pos;
                        self.back_and_forth_forward = false;
                    }
                } else {
                    self.pos -= self.pitch;
                    if self.pos <= 0.0 {
                        self.pos = 0.0;
                        self.back_and_forth_forward = true;
                    }
                }
            }
        }

        self.gain = self.env.next_sample() * gain;

        self.counter += 1;

        if self.counter > self.length {
            self.reset();
        }

        GrainData {
            pos: self.pos,
            gain: self.gain,
            stereo_pos: self.stereo_pos,
        }
    }

    pub fn reset(&mut self) {
        self.active = false;
        self.counter = 0;
        self.gain = 0.0;
        self.env.reset();
    }
}

#[derive(Default, Copy, Clone)]
struct Envelope {
    inc: f64,
    phase: f64,
    sin0: f64,
    sin1: f64,
    dsin: f64,
}

impl Envelope {
    pub fn reset(&mut self) {
        self.phase = 0.0;
        self.sin0 = (self.phase * PI).sin();
        self.sin1 = ((self.phase - self.inc) * PI).sin();
        self.dsin = 2.0 * (self.inc * PI).cos();
    }

    pub fn set_inc(&mut self, inc: f32) {
        self.inc = inc as f64;
        self.reset();
    }

    pub fn next_sample(&mut self) -> f32 {
        let sinx = self.dsin * self.sin0 - self.sin1;
        self.sin1 = self.sin0;
        self.sin0 = sinx;
        sinx as f32
    }
}
