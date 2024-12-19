use nih_plug::nih_error;
use nih_plug::prelude::Editor;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::GenericUi;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState, ViziaTheming};
use triple_buffer::Output;
use std::sync::{Arc, Mutex};

use crate::sampler::DrawData;
use crate::GrainiacParams;
mod waveform;

#[derive(Lens)]
struct Data {
    params: Arc<GrainiacParams>,
}

impl Model for Data {}

pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (300, 450))
}

pub(crate) fn create(
    params: Arc<GrainiacParams>,
    editor_state: Arc<ViziaState>,
    draw_data: Arc<Mutex<Output<DrawData>>>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, _| {
        assets::register_noto_sans_light(cx);
        assets::register_noto_sans_thin(cx);

        if let Err(err) = cx.add_stylesheet(include_style!("src/editor/styles.css")) {
            nih_error!("Failed to load stylesheet: {err:?}")
        }

        Data {
            params: params.clone(),
        }
        .build(cx);

        VStack::new(cx, |cx| {
            GenericUi::new(cx, Data::params).child_top(Pixels(10.0));
        });
        HStack::new(cx, |cx| {
            waveform::Waveform::new(cx, draw_data.clone());
        })
        .min_top(Pixels(30.0))
        .left(Pixels(15.0))
        .right(Pixels(15.0))
        .height(Pixels(100.0))
        .class("waveform");
    })
}
