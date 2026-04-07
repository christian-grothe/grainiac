use crossbeam::channel::Sender;
use nih_plug::nih_error;
use nih_plug::prelude::Editor;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::{create_vizia_editor, ViziaState, ViziaTheming};
use rfd::FileDialog;
use std::sync::{Arc, Mutex};

use crate::editor::widgets::dial::Dial;
use crate::editor::widgets::select::Select;
use crate::editor::widgets::waveform::Waveform;
use crate::{utils, FileMessage, GrainiacParams};
use grainiac_core::{DrawData, Output};

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
            let path_str = path.to_string_lossy().to_string();
            if let Some(samples) = utils::AudioHandler::open(path) {
                if let Ok(mut paths) = self.params.audio_paths.lock() {
                    paths[index] = Some(path_str);
                }
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
        if let Err(err) = cx.add_stylesheet(include_style!("src/editor/styles.css")) {
            nih_error!("Failed to load stylesheet: {err:?}")
        }

        cx.add_font_mem(include_bytes!("editor/JetBrainsMonoNerdFont-Medium.ttf"));
        cx.set_default_font(&["JetBrainsMono Nerd Font"]);

        Data {
            params: params.clone(),
            sender: sender.clone(),
        }
        .build(cx);

        VStack::new(cx, |cx| {
            top_bar(cx);
            VStack::new(cx, |cx| {
                (0..2).for_each(|i| instance_block(cx, draw_data.clone(), i as usize));
            })
            .class("instance-list")
            .child_space(Pixels(28.0))
            .width(Stretch(1.0));
        })
        .class("layout")
        .child_space(Pixels(32.0))
        .width(Stretch(1.0));
    })
}

fn top_bar(cx: &mut Context) {
    HStack::new(cx, |cx| {
        VStack::new(cx, |cx| {
            Label::new(cx, "Grainiac")
                .class("title")
                .text_align(TextAlign::Left);
            Label::new(cx, "Granular Sampler Instrument")
                .class("subtitle")
                .text_align(TextAlign::Left);
        })
        .width(Stretch(1.0))
        .child_space(Pixels(6.0));

        Label::new(cx, "Timerift Audio")
            .class("brand-mark")
            .text_align(TextAlign::Right)
            .width(Stretch(0.5));
    })
    .class("top-bar")
    .child_space(Pixels(12.0))
    .child_left(Pixels(24.0))
    .child_right(Pixels(24.0))
    .width(Stretch(1.0));
}

fn instance_block(
    cx: &mut Context,
    draw_data: Arc<Mutex<Output<Vec<DrawData>>>>,
    index: usize,
) {
    let title = match index {
        0 => "Layer A",
        1 => "Layer B",
        _ => "Layer",
    };

    VStack::new(cx, |cx| {
        HStack::new(cx, |cx| {
            Label::new(cx, title).class("section-heading");
            Button::new(
                cx,
                move |ex| {
                    ex.emit(FileMessage::OpenFileDialog(index));
                },
                |cx| Label::new(cx, "Load Audio"),
            )
            .class("accent-button")
            .height(Pixels(38.0));
        })
        .class("instance-header")
        .child_space(Pixels(12.0));

        waveform_panel(cx, draw_data.clone(), index);

        control_row(cx, index);
        dial_grid(cx, index);
    })
    .class("instance-card")
    .child_space(Pixels(18.0));
}

fn waveform_panel(cx: &mut Context, draw_data: Arc<Mutex<Output<Vec<DrawData>>>>, index: usize) {
    ZStack::new(cx, |cx| {
        Waveform::new(cx, draw_data.clone(), index);
    })
    .class("waveform-panel")
    .height(Pixels(160.0));
}

fn control_row(cx: &mut Context, index: usize) {
    HStack::new(cx, |cx| {
        Select::new(cx, "Grain Dir", 2, Data::params, move |params| {
            &params.instances[index].g_dir
        })
        .class("select-control");

        Select::new(cx, "Play Dir", 2, Data::params, move |params| {
            &params.instances[index].p_dir
        })
        .class("select-control");

        Select::new(cx, "Hold", 2, Data::params, move |params| {
            &params.instances[index].hold
        })
        .class("select-control");
    })
    .class("control-row")
    .child_space(Pixels(18.0));
}

fn dial_grid(cx: &mut Context, index: usize) {
    HStack::new(cx, |cx| {
        VStack::new(cx, |cx| {
            Dial::new(cx, "Loop Start", Data::params, move |params| {
                &params.instances[index].loop_start
            })
            .class("dial-block");
            Dial::new(cx, "Loop End", Data::params, move |params| {
                &params.instances[index].loop_length
            })
            .class("dial-block");
        })
        .class("dial-column")
        .child_space(Pixels(16.0));

        VStack::new(cx, |cx| {
            Dial::new(cx, "Density", Data::params, move |params| {
                &params.instances[index].density
            })
            .class("dial-block");
            Dial::new(cx, "Grain Length", Data::params, move |params| {
                &params.instances[index].grain_length
            })
            .class("dial-block");
        })
        .class("dial-column")
        .child_space(Pixels(16.0));

        VStack::new(cx, |cx| {
            Dial::new(cx, "Play Speed", Data::params, move |params| {
                &params.instances[index].play_speed
            })
            .class("dial-block");
            Dial::new(cx, "Spray", Data::params, move |params| {
                &params.instances[index].spray
            })
            .class("dial-block");
        })
        .class("dial-column")
        .child_space(Pixels(16.0));

        VStack::new(cx, |cx| {
            Dial::new(cx, "Pan", Data::params, move |params| {
                &params.instances[index].pan
            })
            .class("dial-block");
            Dial::new(cx, "Spread", Data::params, move |params| {
                &params.instances[index].spread
            })
            .class("dial-block");
        })
        .class("dial-column")
        .child_space(Pixels(16.0));

        VStack::new(cx, |cx| {
            Dial::new(cx, "Attack", Data::params, move |params| {
                &params.instances[index].attack
            })
            .class("dial-block");
            Dial::new(cx, "Release", Data::params, move |params| {
                &params.instances[index].release
            })
            .class("dial-block");
        })
        .class("dial-column")
        .child_space(Pixels(16.0));

        VStack::new(cx, |cx| {
            Dial::new(cx, "Pitch", Data::params, move |params| {
                &params.instances[index].pitch
            })
            .class("dial-block");
            Dial::new(cx, "Gain", Data::params, move |params| {
                &params.instances[index].gain
            })
            .class("dial-block");
        })
        .class("dial-column")
        .child_space(Pixels(16.0));
    })
    .class("dial-grid")
    .child_space(Pixels(20.0));
}
