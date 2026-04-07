use std::sync::{Arc, Mutex};

use nih_plug_vizia::{
    vizia::{
        binding::Lens,
        context::{Context, DrawContext, EventContext},
        events::Event,
        input::MouseButton,
        vg::{Color, Paint, Path, Solidity},
        view::{Canvas, Handle, View},
        window::WindowEvent,
    },
    widgets::param_base::ParamWidgetBase,
};

use crate::GrainiacParams;
use grainiac_core::{DrawData, Output};

const HANDLE_HITBOX_PX: f32 = 10.0;

#[derive(Copy, Clone, PartialEq, Eq)]
enum DragTarget {
    Start,
    End,
}

pub struct Waveform {
    draw_data: Arc<Mutex<Output<Vec<DrawData>>>>,
    index: usize,
    loop_start_param: ParamWidgetBase,
    loop_length_param: ParamWidgetBase,
    drag_target: Option<DragTarget>,
}

impl Waveform {
    pub fn new<'a, L>(
        cx: &'a mut Context,
        draw_data: Arc<Mutex<Output<Vec<DrawData>>>>,
        params: L,
        index: usize,
    ) -> Handle<'a, Self>
    where
        L: Lens<Target = Arc<GrainiacParams>> + Clone + 'static,
    {
        let loop_start_param = ParamWidgetBase::new(
            cx,
            params.clone(),
            move |params: &Arc<GrainiacParams>| &params.instances[index].loop_start,
        );
        let loop_length_param = ParamWidgetBase::new(
            cx,
            params,
            move |params: &Arc<GrainiacParams>| &params.instances[index].loop_length,
        );

        Self {
            draw_data,
            index,
            loop_start_param,
            loop_length_param,
            drag_target: None,
        }
        .build(cx, |_cx| ())
    }

    fn handle_pointer_position(&self, cx: &EventContext) -> Option<(f32, f32, f32)> {
        let bounds = cx.bounds();
        if bounds.w == 0.0 || bounds.h == 0.0 {
            return None;
        }

        let cursor_x = cx.mouse().cursorx;
        if cursor_x < bounds.x || cursor_x > bounds.x + bounds.w {
            return None;
        }

        let draw_data = self.draw_data.lock().ok()?;
        let draw = draw_data.read();
        let state = &draw[self.index].state;

        let loop_start = state.loop_start;
        let loop_length = state.loop_length;
        let loop_end = (loop_start + loop_length).min(1.0);

        let normalized_pos = ((cursor_x - bounds.x) / bounds.w).clamp(0.0, 1.0);

        Some((normalized_pos, loop_start, loop_end))
    }
}

impl View for Waveform {
    fn element(&self) -> Option<&'static str> {
        Some("waveform")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|window_event, _meta| match window_event {
            WindowEvent::MouseDown(MouseButton::Left) => {
                if let Some((normalized_pos, loop_start, loop_end)) =
                    self.handle_pointer_position(cx)
                {
                    let bounds = cx.bounds();
                    let start_x = bounds.x + bounds.w * loop_start;
                    let end_x = bounds.x + bounds.w * loop_end;
                    let cursor_x = cx.mouse().cursorx;

                    if (cursor_x - start_x).abs() <= HANDLE_HITBOX_PX {
                        let current_length =
                            self.loop_length_param.unmodulated_normalized_value();
                        let new_start =
                            normalized_pos.min(1.0 - current_length).max(0.0);

                        self.drag_target = Some(DragTarget::Start);
                        self.loop_start_param.begin_set_parameter(cx);
                        self.loop_start_param.set_normalized_value(cx, new_start);
                        cx.needs_redraw();
                        cx.capture();
                    } else if (cursor_x - end_x).abs() <= HANDLE_HITBOX_PX {
                        self.drag_target = Some(DragTarget::End);
                        self.loop_length_param.begin_set_parameter(cx);
                        let current_loop_start =
                            self.loop_start_param.unmodulated_normalized_value();
                        let length =
                            (normalized_pos - current_loop_start).clamp(0.0, 1.0 - current_loop_start);
                        self.loop_length_param.set_normalized_value(cx, length);
                        cx.needs_redraw();
                        cx.capture();
                    }
                }
            }
            WindowEvent::MouseMove(_, _) => {
                if let Some(target) = self.drag_target {
                    if let Some((normalized_pos, _loop_start, _loop_end)) =
                        self.handle_pointer_position(cx)
                    {
                        match target {
                            DragTarget::Start => {
                                let current_length =
                                    self.loop_length_param.unmodulated_normalized_value();
                                let new_start =
                                    normalized_pos.min(1.0 - current_length).max(0.0);
                                self.loop_start_param.set_normalized_value(cx, new_start);
                            }
                            DragTarget::End => {
                                let loop_start =
                                    self.loop_start_param.unmodulated_normalized_value();
                                let new_length =
                                    (normalized_pos - loop_start).clamp(0.0, 1.0 - loop_start);
                                self.loop_length_param
                                    .set_normalized_value(cx, new_length.max(0.0));
                            }
                        }
                        cx.needs_redraw();
                    }
                }
            }
            WindowEvent::MouseUp(MouseButton::Left) | WindowEvent::MouseCaptureLost => {
                if let Some(target) = self.drag_target.take() {
                    match target {
                        DragTarget::Start => self.loop_start_param.end_set_parameter(cx),
                        DragTarget::End => self.loop_length_param.end_set_parameter(cx),
                    }
                    cx.release();
                }
            }
            _ => {}
        });
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();
        if bounds.w == 0.0 || bounds.h == 0.0 {
            return;
        }

        let draw_data = self.draw_data.lock().unwrap();
        let draw = draw_data.read();
        let buffer = draw[self.index].buffer.clone();
        let grain_data = draw[self.index].grain_data.clone();
        let mut loop_start = draw[self.index].state.loop_start;
        let mut loop_length = draw[self.index].state.loop_length;

        if self.drag_target.is_some() {
            loop_start = self.loop_start_param.unmodulated_normalized_value();
            loop_length = self.loop_length_param.unmodulated_normalized_value();
        }

        let loop_area = (loop_start, loop_length);

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
                canvas.stroke_path(&path, &paint);
            }
        });

        let paint = Paint::color(Color::rgba(200, 200, 200, 50));
        let mut path = Path::new();
        let x = bounds.w * loop_area.0;
        let mut w = bounds.w * loop_area.1;
        if x + w > bounds.w {
            w = bounds.w - x;
        }
        path.rect(bounds.x + x, bounds.y, w, bounds.h);
        canvas.fill_path(&path, &paint);

        let handle_color = Paint::color(Color::rgb(255, 203, 107));
        let mut start_handle = Path::new();
        start_handle.rect(
            bounds.x + x - 1.0,
            bounds.y,
            2.0,
            bounds.h,
        );
        canvas.fill_path(&start_handle, &handle_color);

        let end_x = (x + w).min(bounds.w);
        let mut end_handle = Path::new();
        end_handle.rect(
            bounds.x + end_x - 1.0,
            bounds.y,
            2.0,
            bounds.h,
        );
        canvas.fill_path(&end_handle, &handle_color);
    }
}
