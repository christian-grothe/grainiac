use ratatui::{
    layout::{Constraint, Direction, Layout},
    text::Span,
    Frame,
};

use crate::{
    state::{PresetMode, State},
    waveform_widget::Waveform,
};

pub fn draw(frame: &mut Frame, state: &mut State) {
    let out_buf = state.out_buf.read();

    let layout_horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Length(100)])
        .split(frame.area());

    let tracks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(10); 5])
        .split(layout_horizontal[0]);

    let track_a = Waveform::from("A", out_buf[0].clone());
    let track_b = Waveform::from("B", out_buf[1].clone());
    let track_c = Waveform::from("C", out_buf[2].clone());
    let track_d = Waveform::from("D", out_buf[3].clone());

    frame.render_widget(track_a, tracks[0]);
    frame.render_widget(track_b, tracks[1]);
    frame.render_widget(track_c, tracks[2]);
    frame.render_widget(track_d, tracks[3]);

    if state.preset_mode == PresetMode::Save {
        let span = Span::from("Press a key to save a new preset");
        frame.render_widget(span, tracks[4]);
    }
}
