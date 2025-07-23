use nih_plug::params::Param;
use nih_plug_vizia::{
    vizia::{
        binding::Lens,
        context::{Context, EventContext},
        events::Event,
        input::MouseButton,
        layout::Units::{self},
        modifiers::{LayoutModifiers, StyleModifiers},
        view::{Handle, View},
        views::{HStack, Label},
        window::WindowEvent,
    },
    widgets::param_base::ParamWidgetBase,
};

pub struct Select {
    param_base: ParamWidgetBase,
    variants: usize,
}

impl View for Select {
    fn element(&self) -> Option<&'static str> {
        Some("select")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|window_event, _meta| match window_event {
            WindowEvent::MouseDown(MouseButton::Left) => {
                let current = self.param_base.unmodulated_normalized_value();
                let next = current + 1.0 / self.variants as f32;
                let new = if next > 1.0 { 0.0 } else { next };
                self.param_base.begin_set_parameter(cx);
                self.param_base.set_normalized_value(cx, new);
            }
            WindowEvent::MouseUp(MouseButton::Left) => {
                self.param_base.end_set_parameter(cx);
            }
            _ => {}
        });
    }
}

impl Select {
    pub fn new<'a, L, Params, P, FMap>(
        cx: &'a mut Context,
        label: &'static str,
        variants: usize,
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
            variants,
        }
        .build(
            cx,
            ParamWidgetBase::build_view(params, params_to_param, move |cx, param_data| {
                let display_value_lens = param_data.make_lens(|param| {
                    param.normalized_value_to_string(param.unmodulated_normalized_value(), true)
                });
                HStack::new(cx, |cx| {
                    Label::new(cx, &format!("{}:  ", label));
                    Label::new(cx, display_value_lens).class("bold");
                })
                .width(Units::Auto);
            }),
        )
    }
}
