use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::{state::State, waveform_widget::Waveform};

pub fn draw(frame: &mut Frame, state: &mut State) {
    let out_buf = state.out_buf.read();

    let layout_vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Min(0),
            Constraint::Length(4 * 7),
            Constraint::Min(0),
        ])
        .split(frame.area());

    let layout_horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Min(0),
            Constraint::Length(100),
            Constraint::Min(0),
        ])
        .split(layout_vertical[1]);

    let tracks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(10); 4])
        .split(layout_horizontal[1]);

    let track_a = Waveform::from("Track A", out_buf[0].clone());
    let track_b = Waveform::from("Track B", out_buf[1].clone());
    let track_c = Waveform::from("Track C", out_buf[2].clone());
    let track_d = Waveform::from("Track D", out_buf[3].clone());

    frame.render_widget(track_a, tracks[0]);
    frame.render_widget(track_b, tracks[1]);
    frame.render_widget(track_c, tracks[2]);
    frame.render_widget(track_d, tracks[3]);
}
