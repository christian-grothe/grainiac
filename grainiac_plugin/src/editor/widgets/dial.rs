use nih_plug::params::Param;
use nih_plug_vizia::{
    vizia::{
        binding::Lens,
        context::Context,
        layout::Units::{Pixels, Stretch},
        modifiers::{LayoutModifiers, TextModifiers},
        style::TextAlign,
        view::{Handle, View},
        views::Label,
    },
    widgets::param_base::ParamWidgetBase,
};

use crate::editor::widgets::dial_base::DialBase;

pub struct Dial {}

impl View for Dial {
    fn element(&self) -> Option<&'static str> {
        Some("dial")
    }
}

impl Dial {
    pub fn new<'a, L, Params, P, FMap>(
        cx: &'a mut Context,
        label: &'static str,
        params: L,
        params_to_param: FMap,
    ) -> Handle<'a, Self>
    where
        L: Lens<Target = Params> + Clone,
        Params: 'static,
        P: Param + 'static,
        FMap: Fn(&Params) -> &P + Copy + 'static,
    {
        Self {}.build(
            cx,
            ParamWidgetBase::build_view(params, params_to_param, move |cx, param_data| {
                let display_value_lens = param_data.make_lens(|param| {
                    param.normalized_value_to_string(param.unmodulated_normalized_value(), true)
                });

                Label::new(cx, label)
                    .width(Stretch(1.0))
                    .bottom(Pixels(10.0))
                    .text_align(TextAlign::Center);
                DialBase::new(cx, params, params_to_param);
                Label::new(cx, display_value_lens)
                    .width(Stretch(1.0))
                    .bottom(Pixels(10.0))
                    .text_align(TextAlign::Center);
            }),
        )
    }
}
