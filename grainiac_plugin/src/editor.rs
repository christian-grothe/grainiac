use crossbeam::channel::Sender;
use nih_plug::nih_error;
use nih_plug::prelude::Editor;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState, ViziaTheming};
use rfd::FileDialog;
use std::sync::{Arc, Mutex};

use crate::editor::widgets::dial::Dial;
use crate::editor::widgets::select::Select;
use crate::editor::widgets::waveform::Waveform;
use crate::{utils, FileMessage, GrainiacParams};
use grainiac_core::{DrawData, Output, INSTANCE_NUM};

mod widgets;

#[derive(Lens)]
struct Data {
    params: Arc<GrainiacParams>,
    sender: Arc<Sender<FileMessage>>,
}

impl Data {
    fn open_file_dialog(&self, index: usize) {
        let file = FileDialog::new()
            .add_filter("audio", &["wav"])
            .set_directory("/")
            .pick_file();

        if let Some(path) = file {
            if let Some(samples) = utils::AudioHandler::open(path) {
                self.sender
                    .send(FileMessage::LoadAudio(samples, index))
                    .unwrap();
            }
        }
    }
}

impl Model for Data {
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        event.map(|data_event, _meta| match data_event {
            FileMessage::OpenFileDialog(index) => {
                self.open_file_dialog(*index);
            }
            _ => {}
        });
    }
}

pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (800, 900))
}

pub(crate) fn create(
    params: Arc<GrainiacParams>,
    editor_state: Arc<ViziaState>,
    draw_data: Arc<Mutex<Output<Vec<DrawData>>>>,
    sender: Arc<Sender<FileMessage>>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, _| {
        assets::register_noto_sans_light(cx);
        assets::register_noto_sans_thin(cx);

        if let Err(err) = cx.add_stylesheet(include_style!("src/editor/styles.css")) {
            nih_error!("Failed to load stylesheet: {err:?}")
        }

        Data {
            params: params.clone(),
            sender: sender.clone(),
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
            //.font_family(vec![FamilyOwned::Name(String::from(assets::NOTO_SANS))])
            //.font_weight(FontWeightKeyword::Thin)
            .width(Stretch(1.0))
            .font_size(25.0)
            .text_align(TextAlign::Right);
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
    ZStack::new(cx, |cx| {
        Button::new(
            cx,
            move |ex| {
                ex.emit(FileMessage::OpenFileDialog(index));
            },
            |cx| Label::new(cx, "open"),
        )
        .position_type(PositionType::SelfDirected)
        .z_index(10)
        .color(Color::white())
        .border_width(Pixels(0.0))
        .background_color(Color::rgb(150, 100, 100));

        Waveform::new(cx, draw_data.clone(), index);
    })
    .left(Pixels(15.0))
    .right(Pixels(15.0))
    .top(Pixels(15.0))
    .bottom(Pixels(25.0))
    .height(Pixels(100.0))
    .class("waveform");
}

fn instance(cx: &mut Context, index: usize) {
    HStack::new(cx, |cx| {
        Select::new(cx, "grain dir", 3, Data::params, move |params| {
            &params.instances[index].g_dir
        })
        .class("button")
        .width(Units::Auto)
        .left(Pixels(15.0))
        .right(Pixels(15.0));

        Select::new(cx, "play dir", 3, Data::params, move |params| {
            &params.instances[index].p_dir
        })
        .class("button")
        .width(Units::Auto);
    })
    .height(Pixels(30.0));

    HStack::new(cx, |cx| {
        VStack::new(cx, |cx| {
            Dial::new(cx, "loop start", Data::params, move |params| {
                &params.instances[index].loop_start
            });
            Dial::new(cx, "loop end", Data::params, move |params| {
                &params.instances[index].loop_length
            });
        })
        .text_align(TextAlign::Center);

        VStack::new(cx, |cx| {
            Dial::new(cx, "dens", Data::params, move |params| {
                &params.instances[index].density
            });
            Dial::new(cx, "grain_length", Data::params, move |params| {
                &params.instances[index].grain_length
            });
        });

        VStack::new(cx, |cx| {
            Dial::new(cx, "play speed", Data::params, move |params| {
                &params.instances[index].play_speed
            });
            Dial::new(cx, "spray", Data::params, move |params| {
                &params.instances[index].spray
            });
        });

        VStack::new(cx, |cx| {
            Dial::new(cx, "pan", Data::params, move |params| {
                &params.instances[index].pan
            });
            Dial::new(cx, "spread", Data::params, move |params| {
                &params.instances[index].spread
            });
        });

        VStack::new(cx, |cx| {
            Dial::new(cx, "att", Data::params, move |params| {
                &params.instances[index].attack
            });
            Dial::new(cx, "rel", Data::params, move |params| {
                &params.instances[index].release
            });
        });

        VStack::new(cx, |cx| {
            Dial::new(cx, "pitch", Data::params, move |params| {
                &params.instances[index].pitch
            });
            Dial::new(cx, "gain", Data::params, move |params| {
                &params.instances[index].gain
            });
        });
    })
    .text_align(TextAlign::Center)
    .left(Pixels(15.0))
    .right(Pixels(15.0));
}
