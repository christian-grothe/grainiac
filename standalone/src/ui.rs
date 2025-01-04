use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Span,
    widgets::Block,
    Frame,
};

use crate::{state::State, waveform_widget::Waveform};

pub fn draw(frame: &mut Frame, state: &mut State) {
    let out_buf = state.out_buf.read();

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(Constraint::from_percentages([25, 50, 25]))
        .split(frame.area());

    let center = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(Constraint::from_percentages([25, 50, 25]))
        .split(layout[1]);

    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints(Constraint::from_percentages([5, 95]))
        .split(center[1]);

    let canvas_areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints(Constraint::from_percentages([25, 25, 25, 25]))
        .split(areas[1]);

    let title = Block::new().title(Span::styled("Grainiac", Style::default().fg(Color::Yellow)));

    let track_a = Waveform::from("Track A", out_buf[0].clone());
    let track_b = Waveform::from("Track B", out_buf[1].clone());
    let track_c = Waveform::from("Track C", out_buf[2].clone());
    let track_d = Waveform::from("Track D", out_buf[3].clone());

    frame.render_widget(title, areas[0]);
    frame.render_widget(track_a, canvas_areas[0]);
    frame.render_widget(track_b, canvas_areas[1]);
    frame.render_widget(track_c, canvas_areas[2]);
    frame.render_widget(track_d, canvas_areas[3]);
}
