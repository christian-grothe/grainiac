use std::sync::{Arc, Mutex};

use nih_plug_vizia::vizia::{
    context::{Context, DrawContext},
    vg::{Color, Paint, Path, Solidity},
    view::{Canvas, Handle, View},
};

use grainiac_core::{DrawData, Output};

pub struct Waveform {
    draw_data: Arc<Mutex<Output<Vec<DrawData>>>>,
    index: usize,
}

impl Waveform {
    pub fn new(
        cx: &mut Context,
        draw_data: Arc<Mutex<Output<Vec<DrawData>>>>,
        index: usize,
    ) -> Handle<Self> {
        Self { draw_data, index }.build(cx, |_cx| ())
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

        let (buffer, grain_data, play_heads, loop_area) = {
            let mut draw_data = self.draw_data.lock().unwrap();
            let layer = &draw_data.read()[self.index];
            (
                layer.buffer.clone(),
                layer.grain_data.clone(),
                layer.play_heads.clone(),
                (layer.state.loop_start, layer.state.loop_length),
            )
        };

        let mut baseline = Paint::color(Color::rgba(55, 82, 118, 140));
        baseline.set_line_width(1.0);
        let mut baseline_path = Path::new();
        baseline_path.move_to(bounds.x, bounds.y + bounds.h * 0.5);
        baseline_path.line_to(bounds.x + bounds.w, bounds.y + bounds.h * 0.5);
        canvas.stroke_path(&baseline_path, &baseline);

        let mut bars_paint = Paint::color(Color::rgba(106, 158, 211, 180));
        let mut path = Path::new();

        for (i, bar) in buffer.iter().enumerate() {
            path.rect(
                bounds.x + bounds.w * i as f32 / buffer.len() as f32,
                (bounds.y + bounds.h / 2.0) - (bounds.h * bar / 2.0),
                2.0,
                bounds.h * bar,
            );
        }

        canvas.fill_path(&path, &bars_paint);

        let mut grains_paint = Paint::color(Color::hex("#4FDCC6"));
        grains_paint.set_line_width(2.0);

        grain_data.iter().for_each(|data| {
            if let Some(data) = data {
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
                canvas.stroke_path(&path, &grains_paint);
            }
        });

        let mut playhead_paint = Paint::color(Color::rgba(125, 248, 225, 160));
        playhead_paint.set_line_width(2.0);
        for head in play_heads.iter().flatten() {
            let mut path = Path::new();
            let x = bounds.x + bounds.w * head;
            path.move_to(x, bounds.y);
            path.line_to(x, bounds.y + bounds.h);
            canvas.stroke_path(&path, &playhead_paint);
        }

        let mut path = Path::new();
        let x = bounds.w * loop_area.0;
        let mut w = bounds.w * loop_area.1;
        if x + w > bounds.w {
            w = bounds.w - x;
        }
        path.rect(bounds.x + x, bounds.y, w, bounds.h);

        let mut loop_paint = Paint::color(Color::rgba(79, 220, 198, 45));
        canvas.fill_path(&path, &loop_paint);
        let mut loop_outline = Paint::color(Color::rgba(125, 248, 225, 90));
        loop_outline.set_line_width(1.5);
        canvas.stroke_path(&path, &loop_outline);
    }
}
