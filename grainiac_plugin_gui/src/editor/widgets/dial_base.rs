use std::f32::consts::PI;

use nih_plug::params::Param;
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

pub struct DialBase {
    param_base: ParamWidgetBase,
    is_clicked: bool,
    last_mouse_y: f32,
    drag_sensitivity: f32,
}

impl View for DialBase {
    fn element(&self) -> Option<&'static str> {
        Some("dial")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|window_event, _meta| match window_event {
            WindowEvent::MouseDown(MouseButton::Left) => {
                cx.capture();
                self.is_clicked = true;
                // Store the initial mouse position when starting to drag
                self.last_mouse_y = cx.mouse().cursory;
                self.param_base.begin_set_parameter(cx);
            }
            WindowEvent::MouseUp(MouseButton::Left) => {
                cx.release();

                self.param_base.end_set_parameter(cx);
                self.is_clicked = false;
            }
            WindowEvent::MouseDoubleClick(MouseButton::Left) => {
                let default_val = self.param_base.default_normalized_value();

                self.param_base.begin_set_parameter(cx);
                self.param_base.set_normalized_value(cx, default_val);
                self.param_base.end_set_parameter(cx);
            }
            WindowEvent::MouseMove(_x, y) => {
                if self.is_clicked {
                    // Calculate the delta (difference) from last mouse position
                    let delta_y = self.last_mouse_y - y;

                    // Get current parameter value
                    let current_value = self.param_base.unmodulated_normalized_value();

                    // Apply delta with sensitivity
                    let new_value =
                        (current_value + delta_y * self.drag_sensitivity).clamp(0.0, 1.0);

                    // Set the new value
                    self.param_base.set_normalized_value(cx, new_value);

                    // Update last mouse position for next frame
                    self.last_mouse_y = *y;
                }
            }
            _ => {}
        });
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let val = self.param_base.unmodulated_normalized_value();

        let bounds = cx.bounds();
        if bounds.w == 0.0 || bounds.h == 0.0 {
            return;
        }

        let center_x = bounds.x + bounds.w * 0.5;
        let center_y = bounds.y + bounds.h * 0.5;

        let radius = bounds.w.min(bounds.h) * 0.45;

        let start_angle = PI * 0.75;
        let end_angle = PI * 2.25;

        let mut track = Paint::color(Color::rgba(82, 116, 154, 120));
        track.set_line_width(6.0);

        let mut path = Path::new();
        path.arc(
            center_x,
            center_y,
            radius,
            start_angle,
            end_angle,
            Solidity::Hole,
        );

        canvas.stroke_path(&path, &track);

        let mut paint = Paint::color(Color::hex("#4FDCC6"));
        paint.set_line_width(6.0);

        let angle = start_angle + (end_angle - start_angle) * val;

        let line_to_x = center_x + radius * angle.cos();
        let line_to_y = center_y + radius * angle.sin();

        let mut arc_path = Path::new();
        arc_path.arc(
            center_x,
            center_y,
            radius,
            start_angle,
            angle,
            Solidity::Hole,
        );

        canvas.stroke_path(&arc_path, &paint);

        let mut indicator = Paint::color(Color::hex("#7DF8E1"));
        indicator.set_line_width(2.0);

        let mut pointer = Path::new();
        pointer.move_to(center_x, center_y);
        pointer.line_to(line_to_x, line_to_y);

        canvas.stroke_path(&pointer, &indicator);

        let mut hub = Paint::color(Color::rgba(9, 18, 28, 200));
        let mut hub_path = Path::new();
        hub_path.arc(center_x, center_y, radius * 0.35, 0.0, PI * 2.0, Solidity::Solid);
        canvas.fill_path(&hub_path, &hub);

        let mut hub_ring = Paint::color(Color::rgba(125, 248, 225, 45));
        hub_ring.set_line_width(2.0);
        let mut hub_ring_path = Path::new();
        hub_ring_path.arc(center_x, center_y, radius * 0.38, 0.0, PI * 2.0, Solidity::Hole);
        canvas.stroke_path(&hub_ring_path, &hub_ring);
    }
}

impl DialBase {
    pub fn new<'a, L, Params, P, FMap>(
        cx: &'a mut Context,
        params: L,
        params_to_param: FMap,
    ) -> Handle<'a, Self>
    where
        L: Lens<Target = Params> + Clone,
        Params: 'static,
        P: Param + 'static,
        FMap: Fn(&Params) -> &P + Copy + 'static,
    {
        Self {
            param_base: ParamWidgetBase::new(cx, params, params_to_param),
            is_clicked: false,
            last_mouse_y: 0.0,
            drag_sensitivity: 0.01, // Adjust this value to control sensitivity
        }
        .build(cx, |_| {})
    }
}
