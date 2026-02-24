use crate::{INSTANCE_NUM, widgets::track_widget::Track};
use baseview::{
    Event, EventStatus, PhySize, Window, WindowEvent, WindowHandle, WindowHandler,
    WindowOpenOptions, WindowScalePolicy,
};
use crossbeam::{atomic::AtomicCell, channel::Sender};
use grainiac_core::{DrawData, Output};
use keyboard_types::{Key, KeyState};
use nih_plug::{
    editor::Editor,
    params::persist::PersistentField,
    prelude::{GuiContext, ParentWindowHandle},
};
use ratatui::{
    Terminal,
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::Stylize,
    widgets::{Block, Borders},
};
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use soft_ratatui::{
    EmbeddedGraphics, SoftBackend,
    embedded_graphics_unicodefonts::{
        mono_8x13_atlas, mono_8x13_bold_atlas, mono_8x13_italic_atlas,
    },
};
use std::{
    num::NonZeroU32,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use crate::{FileMessage, GrainiacParams, utils};

const FONT_W: u32 = 8;
const FONT_H: u32 = 13;

#[derive(Debug, Serialize, Deserialize)]
pub struct RatatuiState {
    #[serde(skip)]
    open: AtomicBool,

    #[serde(with = "nih_plug::params::persist::serialize_atomic_cell")]
    size: AtomicCell<(u32, u32)>,
}

impl<'a> PersistentField<'a, RatatuiState> for Arc<RatatuiState> {
    fn set(&self, new_value: RatatuiState) {
        self.size.store(new_value.size.load());
    }

    fn map<F, R>(&self, f: F) -> R
    where
        F: Fn(&RatatuiState) -> R,
    {
        f(self)
    }
}

impl Default for RatatuiState {
    fn default() -> Self {
        Self {
            open: AtomicBool::new(false),
            size: AtomicCell::new((850, 200 * INSTANCE_NUM as u32)),
        }
    }
}

pub struct RatatuiEditor {
    pub state: Arc<RatatuiState>,
    pub params: Arc<GrainiacParams>,
    pub draw_data: Arc<Mutex<Output<Vec<DrawData>>>>,
    pub sender: Arc<Sender<FileMessage>>,
}

impl Editor for RatatuiEditor {
    fn spawn(
        &self,
        parent: ParentWindowHandle,
        _context: Arc<dyn GuiContext>,
    ) -> Box<dyn std::any::Any + Send> {
        let (width, height) = self.state.size.load();

        let state = self.state.clone();
        let params = self.params.clone();
        let draw_data = self.draw_data.clone();
        let sender = self.sender.clone();

        let window = Window::open_parented(
            &parent,
            WindowOpenOptions {
                title: "Window Test".into(),
                size: baseview::Size::new(width as f64, height as f64),
                scale: WindowScalePolicy::ScaleFactor(1.0),
            },
            move |window: &mut Window| -> RatatuiWindowHandler {
                RatatuiWindowHandler::new(state.clone(), window, params, draw_data, sender)
            },
        );

        self.state.open.store(true, Ordering::Relaxed);

        Box::new(RatatuiEditorHandle {
            state: self.state.clone(),
            window,
        })
    }

    fn size(&self) -> (u32, u32) {
        self.state.size.load()
    }

    fn set_scale_factor(&self, _factor: f32) -> bool {
        true
    }

    fn param_value_changed(&self, _id: &str, _normalized_value: f32) {}

    fn param_modulation_changed(&self, _id: &str, _modulation_offset: f32) {}

    fn param_values_changed(&self) {}
}

struct RatatuiEditorHandle {
    state: Arc<RatatuiState>,
    window: WindowHandle,
}

unsafe impl Send for RatatuiEditorHandle {}

impl Drop for RatatuiEditorHandle {
    fn drop(&mut self) {
        self.window.close();
        self.state
            .open
            .store(false, std::sync::atomic::Ordering::Release);
    }
}

#[allow(dead_code)]
struct RatatuiWindowHandler {
    _state: Arc<RatatuiState>,
    _ctx: softbuffer::Context,
    surface: softbuffer::Surface,
    current_size: PhySize,
    terminal: Terminal<SoftBackend<EmbeddedGraphics>>,
    params: Arc<GrainiacParams>,
    draw_data: Arc<Mutex<Output<Vec<DrawData>>>>,
    damaged: bool,
    sender: Arc<Sender<FileMessage>>,
}

impl RatatuiWindowHandler {
    fn new(
        state: Arc<RatatuiState>,
        window: &mut Window,
        params: Arc<GrainiacParams>,
        draw_data: Arc<Mutex<Output<Vec<DrawData>>>>,
        sender: Arc<Sender<FileMessage>>,
    ) -> Self {
        let ctx = unsafe { softbuffer::Context::new(window) }.unwrap();
        let mut surface = unsafe { softbuffer::Surface::new(&ctx, window) }.unwrap();

        let (width, height) = state.size.load();

        state.open.store(true, std::sync::atomic::Ordering::Release);

        let font_regular = mono_8x13_atlas();
        let font_italic = mono_8x13_italic_atlas();
        let font_bold = mono_8x13_bold_atlas();

        // SoftBackend takes character columns/rows, not pixel dimensions.
        // The font atlas is 8px wide × 13px tall per character cell.
        let cols = (width / FONT_W).max(1) as u16;
        let rows = (height / FONT_H).max(1) as u16;

        let backend = SoftBackend::<EmbeddedGraphics>::new(
            cols,
            rows,
            font_regular,
            Some(font_bold),
            Some(font_italic),
        );

        // Resize softbuffer to the actual pixel dimensions the backend will produce.
        let pixel_w = cols as u32 * FONT_W;
        let pixel_h = rows as u32 * FONT_H;
        surface
            .resize(
                NonZeroU32::new(pixel_w).unwrap(),
                NonZeroU32::new(pixel_h).unwrap(),
            )
            .unwrap();

        let terminal = Terminal::new(backend).unwrap();

        Self {
            _state: state,
            _ctx: ctx,
            surface,
            current_size: PhySize::new(pixel_w, pixel_h),
            damaged: true,
            params,
            terminal,
            draw_data,
            sender: sender.clone(),
        }
    }
}

impl WindowHandler for RatatuiWindowHandler {
    fn on_frame(&mut self, _window: &mut Window) {
        let mut buf = self.surface.buffer_mut().unwrap();

        self.terminal
            .draw(|f| {
                let area = f.area();

                let binding = self.draw_data.clone();
                let mut binding = binding.lock().unwrap();
                let draw_data = binding.read();

                let block = Block::default()
                    .title("Grainiac")
                    .borders(Borders::ALL)
                    .on_dark_gray();

                let inner = block.inner(area);

                f.render_widget(block, area);

                let layout_horizontal = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(vec![Constraint::Length(100)])
                    .flex(Flex::Center)
                    .split(inner);

                let layout = Layout::default()
                    .direction(Direction::Vertical)
                    .flex(Flex::Center)
                    .constraints(vec![Constraint::Length(10); INSTANCE_NUM])
                    .split(layout_horizontal[0]);

                for i in 0..INSTANCE_NUM {
                    let track = Track::from(&(i + 1).to_string(), draw_data[i].clone());
                    f.render_widget(track, layout[i]);
                }
            })
            .unwrap();

        let new_data: Vec<u32> = self
            .terminal
            .backend()
            .get_pixmap_data()
            .chunks(3)
            .map(|px| {
                let r = px[0] as u32;
                let g = px[1] as u32;
                let b = px[2] as u32;
                0xFF000000 | (r << 16) | (g << 8) | b
            })
            .collect();

        if buf.len() == new_data.len() {
            buf.copy_from_slice(new_data.as_slice());
        }

        buf.present().unwrap();
    }

    fn on_event(&mut self, _window: &mut Window, event: Event) -> EventStatus {
        match event {
            Event::Window(WindowEvent::Resized(info)) => {
                let new_size = info.physical_size();

                let cols = (new_size.width / FONT_W).max(1);
                let rows = (new_size.height / FONT_H).max(1);
                let pixel_w = cols * FONT_W;
                let pixel_h = rows * FONT_H;

                if let (Some(pw), Some(ph)) = (NonZeroU32::new(pixel_w), NonZeroU32::new(pixel_h)) {
                    self.surface.resize(pw, ph).unwrap();
                    self.current_size = PhySize::new(pixel_w, pixel_h);
                    self._state.size.store((new_size.width, new_size.height));

                    let cols16 = cols as u16;
                    let rows16 = rows as u16;
                    // Resize the SoftBackend pixel buffer first, then sync the Terminal viewport.
                    self.terminal.backend_mut().resize(cols16, rows16);
                    let _ = self.terminal.resize(Rect::new(0, 0, cols16, rows16));

                    self.damaged = true;
                }
            }
            Event::Mouse(_e) => {
                self.damaged = true;
                //println!("Mouse event: {:?}", e);
            }
            Event::Keyboard(e) => {
                if e.state == KeyState::Down {
                    match e.key {
                        Key::Character(ref s) if s == "1" => {
                            let file = FileDialog::new()
                                .add_filter("audio", &["wav"])
                                .set_directory("/")
                                .pick_file();

                            if let Some(path) = file {
                                if let Some(samples) = utils::AudioHandler::open(path) {
                                    self.sender
                                        .send(FileMessage::LoadAudio(samples, 0))
                                        .unwrap();
                                }
                            }
                        }
                        Key::Character(ref s) if s == "2" => {
                            let file = FileDialog::new()
                                .add_filter("audio", &["wav"])
                                .set_directory("/")
                                .pick_file();

                            if let Some(path) = file {
                                if let Some(samples) = utils::AudioHandler::open(path) {
                                    self.sender
                                        .send(FileMessage::LoadAudio(samples, 1))
                                        .unwrap();
                                }
                            }
                        }
                        _ => {}
                    }
                    //println!("Keyboard event: {:?}", e);
                }
            }
            _ => {}
        }

        EventStatus::Captured
    }
}
