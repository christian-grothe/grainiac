use ratatui::{
    layout::{Constraint, Direction, Flex, Layout},
    text::Span,
    widgets::{Block, Paragraph},
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
        .flex(Flex::Center)
        .constraints(vec![Constraint::Length(100)])
        .split(frame.area());

    let layout_vertical = Layout::default()
        .direction(Direction::Vertical)
        .flex(Flex::Center)
        .constraints(vec![
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(1),
        ])
        .split(layout_horizontal[0]);

    let layout_menu = Layout::default()
        .direction(Direction::Vertical)
        .flex(Flex::Center)
        .constraints(vec![Constraint::Length(10)])
        .split(layout_horizontal[0]);

    let popup = Paragraph::new("Popup content").block(Block::bordered().title("Popup"));

    let track_a = Waveform::from("A", out_buf[0].clone());
    let track_b = Waveform::from("B", out_buf[1].clone());
    let track_c = Waveform::from("C", out_buf[2].clone());
    let track_d = Waveform::from("D", out_buf[3].clone());

    if state.show_menu {
        frame.render_widget(popup, layout_menu[0]);
    } else {
        frame.render_widget(track_a, layout_vertical[0]);
        frame.render_widget(track_b, layout_vertical[1]);
        frame.render_widget(track_c, layout_vertical[2]);
        frame.render_widget(track_d, layout_vertical[3]);
    }

    if state.preset_mode == PresetMode::Save {
        let span = Span::from("Press a key to save a new preset");
        frame.render_widget(span, layout_vertical[4]);
    }
}
