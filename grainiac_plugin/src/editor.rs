use nih_plug::nih_error;
use nih_plug::prelude::Editor;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::*;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState, ViziaTheming};
use std::sync::{Arc, Mutex};

use crate::GrainiacParams;
use grainiac_core::{DrawData, Output, INSTANCE_NUM};
mod waveform;

#[derive(Lens)]
struct Data {
    params: Arc<GrainiacParams>,
}

impl Model for Data {}

pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (800, 1000))
}

pub(crate) fn create(
    params: Arc<GrainiacParams>,
    editor_state: Arc<ViziaState>,
    draw_data: Arc<Mutex<Output<Vec<DrawData>>>>,
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
            top_bar(cx);
            (0..INSTANCE_NUM).for_each(|i| instace_waveform(cx, draw_data.clone(), i as usize));
        });
    })
}

fn top_bar(cx: &mut Context) {
    HStack::new(cx, |cx| {
        Label::new(cx, "Grainiac")
            .font_family(vec![FamilyOwned::Name(String::from(assets::NOTO_SANS))])
            .font_weight(FontWeightKeyword::Thin)
            .font_size(25.0);
    })
    .left(Pixels(15.0))
    .top(Pixels(10.0))
    .right(Pixels(15.0))
    .height(Pixels(50.0))
    .text_align(TextAlign::Right)
    .width(Stretch(1.0));
}

fn instace_waveform(cx: &mut Context, draw_data: Arc<Mutex<Output<Vec<DrawData>>>>, index: usize) {
    instance(cx, index);
    waveform(cx, draw_data.clone(), index);
}

fn waveform(cx: &mut Context, draw_data: Arc<Mutex<Output<Vec<DrawData>>>>, index: usize) {
    HStack::new(cx, |cx| {
        waveform::Waveform::new(cx, draw_data.clone(), index);
    })
    .left(Pixels(15.0))
    .right(Pixels(15.0))
    .top(Pixels(15.0))
    .bottom(Pixels(25.0))
    .class("waveform");
}

fn instance(cx: &mut Context, index: usize) {
    VStack::new(cx, |cx| {
        HStack::new(cx, |cx| {
            VStack::new(cx, |cx| {
                Label::new(cx, "Loop Start");
                ParamSlider::new(cx, Data::params, move |params| {
                    &params.instances[index].loop_start
                })
                .bottom(Pixels(10.0))
                .set_style(ParamSliderStyle::FromLeft);
                Label::new(cx, "Loop Length");
                ParamSlider::new(cx, Data::params, move |params| {
                    &params.instances[index].loop_length
                })
                .set_style(ParamSliderStyle::FromLeft);
            });
            VStack::new(cx, |cx| {
                Label::new(cx, "Play Speed");
                ParamSlider::new(cx, Data::params, move |params| {
                    &params.instances[index].play_speed
                })
                .bottom(Pixels(10.0))
                .set_style(ParamSliderStyle::FromLeft);
                Label::new(cx, "Density");
                ParamSlider::new(cx, Data::params, move |params| {
                    &params.instances[index].density
                })
                .set_style(ParamSliderStyle::FromLeft);
            });
            VStack::new(cx, |cx| {
                Label::new(cx, "Spray");
                ParamSlider::new(cx, Data::params, move |params| {
                    &params.instances[index].spray
                })
                .bottom(Pixels(10.0))
                .set_style(ParamSliderStyle::FromLeft);
                Label::new(cx, "Grain Length");
                ParamSlider::new(cx, Data::params, move |params| {
                    &params.instances[index].grain_length
                })
                .set_style(ParamSliderStyle::FromLeft);
            });
            VStack::new(cx, |cx| {
                Label::new(cx, "Attack");
                ParamSlider::new(cx, Data::params, move |params| {
                    &params.instances[index].attack
                })
                .bottom(Pixels(10.0))
                .set_style(ParamSliderStyle::FromLeft);
                Label::new(cx, "Release");
                ParamSlider::new(cx, Data::params, move |params| {
                    &params.instances[index].release
                })
                .set_style(ParamSliderStyle::FromLeft);
            });
            VStack::new(cx, |cx| {
                Label::new(cx, "Pan");
                ParamSlider::new(cx, Data::params, move |params| {
                    &params.instances[index].pan
                })
                .bottom(Pixels(10.0))
                .set_style(ParamSliderStyle::FromLeft);
                Label::new(cx, "Spread");
                ParamSlider::new(cx, Data::params, move |params| {
                    &params.instances[index].spread
                })
                .set_style(ParamSliderStyle::FromLeft);
            });
            VStack::new(cx, |cx| {
                Label::new(cx, "Pitch");
                ParamSlider::new(cx, Data::params, move |params| {
                    &params.instances[index].pitch
                })
                .bottom(Pixels(10.0))
                .set_style(ParamSliderStyle::FromLeft);
                Label::new(cx, "Gain");
                ParamSlider::new(cx, Data::params, move |params| {
                    &params.instances[index].gain
                })
                .set_style(ParamSliderStyle::FromLeft);
            });
        });
    })
    .left(Pixels(15.0))
    .right(Pixels(15.0));
}
