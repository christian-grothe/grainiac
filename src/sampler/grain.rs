use std::f64::consts::PI;

#[derive(Default)]
pub struct Grain {
    env: Envelope,
    pub active: bool,
    length: usize,
    counter: usize,
    pos: f32,
    inc: f32,
    gain: f32,
    stereo_pos: f32,
}

impl Grain {
    pub fn activate(&mut self, length: usize, start_pos: f32, pitch: f32, buffer_size: usize) {
        self.active = true;
        self.pos = start_pos;
        self.length = length;
        self.env.set_inc(1.0 / length as f32);
        self.inc = pitch / buffer_size as f32;
        self.stereo_pos = rand::random::<f32>() * 2.0 - 1.0;
    }

    pub fn update(&mut self, gain: f32) -> (f32, f32, f32) {
        self.pos += self.inc;

        if self.pos >= 1.0 {
            self.pos = 0.0;
        }

        self.gain = self.env.next_sample() * gain;

        self.counter += 1;

        if self.counter > self.length {
            self.reset();
        }

        (self.pos, self.gain, self.stereo_pos)
    }

    pub fn reset(&mut self) {
        self.active = false;
        self.counter = 0;
        self.gain = 0.0;
        self.env.reset();
    }
}

#[derive(Default)]
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
