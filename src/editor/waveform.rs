use std::sync::{Arc, Mutex};

use nih_plug_vizia::vizia::{
    context::{Context, DrawContext},
    vg::{Color, Paint, Path, Solidity},
    view::{Canvas, Handle, View},
};
use triple_buffer::Output;

use crate::sampler::DrawData;

pub struct Waveform {
    draw_data: Arc<Mutex<Output<DrawData>>>,
}

impl Waveform {
    pub fn new(cx: &mut Context, draw_data: Arc<Mutex<Output<DrawData>>>) -> Handle<Self> {
        Self { draw_data }.build(cx, |_cx| ())
    }
}

impl View for Waveform {
    fn element(&self) -> Option<&'static str> {
        Some("waveform")
    }
    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();
        if bounds.w == 0.0 || bounds.h == 0.0 {
            return;
        }

        let mut draw_data = self.draw_data.lock().unwrap();
        let buffer = draw_data.read().buffer.clone();
        let voice_data = draw_data.read().voice_data.clone();

        let paint = Paint::color(Color::rgb(200, 200, 200));
        let mut path = Path::new();

        for (i, bar) in buffer.iter().enumerate() {
            path.rect(
                bounds.x + bounds.w * i as f32 / buffer.len() as f32,
                (bounds.y + bounds.h / 2.0) - (bounds.h * bar / 2.0),
                2.0,
                bounds.h * bar,
            );
        }

        canvas.fill_path(&path, &paint);

        let paint = Paint::color(Color::hex("#F6EABE"));

        voice_data.iter().for_each(|data| {
            let mut path = Path::new();
            let y = (data.2 + 1.0) / 2.0;
            path.arc(
                bounds.x + bounds.w * data.0,
                bounds.y + bounds.h * y,
                1.0 + 5.0 * data.1,
                0.0,
                2.0 * std::f32::consts::PI,
                Solidity::Hole,
            );
            canvas.stroke_path(&path, &paint);
        });
    }
}
