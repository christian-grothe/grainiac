use std::{env, fs};

use ratatui::{
    layout::{Constraint, Direction, Flex, Layout},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, List, ListDirection, Paragraph},
    Frame,
};

use crate::{
    state::{NumMode, State, View},
    waveform_widget::Track,
};

pub fn draw(frame: &mut Frame, state: &mut State) {
    match state.view {
        View::Main => render_main_view(frame, state), // render main view,
        View::Preset => render_preset_view(frame, state), // render preset view,
        View::Audio => render_audio_view(frame, state), // render audio view
    }
}

fn render_main_view(frame: &mut Frame, state: &mut State) {
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
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(1),
        ])
        .split(layout_horizontal[0]);

    let track_a = Track::from("A", out_buf[0].clone());
    let track_b = Track::from("B", out_buf[1].clone());
    let track_c = Track::from("C", out_buf[2].clone());
    let track_d = Track::from("D", out_buf[3].clone());

    frame.render_widget(track_a, layout_vertical[2]);
    frame.render_widget(track_b, layout_vertical[3]);
    frame.render_widget(track_c, layout_vertical[4]);
    frame.render_widget(track_d, layout_vertical[5]);

    let span = match state.num_mode {
        NumMode::LoadPreset => Span::from("Press 0 - 9 to LOAD a PRESET"),
        NumMode::SavePreset => Span::from("Press 0 - 9 to SAVE a PRESET"),
        NumMode::LoadAudio => Span::from("Press 0 - 9 to LOAD an AUDIO"),
        NumMode::SaveAudio => Span::from("Press 0 - 9 to SAVE an AUDIO"),
    };

    frame.render_widget(span, layout_vertical[6]);

    let peak = (out_buf[0].input_peak * 15.0) as usize;
    let bar = Span::from(">".repeat(peak));
    let label = Span::from("Input  ");
    let line = Line::from(vec![label, bar]);

    frame.render_widget(line, layout_vertical[0]);

    let peak = (out_buf[0].output_peak * 15.0) as usize;
    let bar = Span::from(">".repeat(peak));
    let label = Span::from("Output ");
    let line = Line::from(vec![label, bar]);

    frame.render_widget(line, layout_vertical[1]);
}

fn render_preset_view(frame: &mut Frame, _state: &mut State) {
    let layout_horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .flex(Flex::Center)
        .constraints(vec![Constraint::Length(100)])
        .split(frame.area());

    let popup = Paragraph::new("TBD").block(Block::bordered().title("Presets"));

    let layout_menu = Layout::default()
        .direction(Direction::Vertical)
        .flex(Flex::Center)
        .constraints(vec![Constraint::Length(10)])
        .split(layout_horizontal[0]);

    frame.render_widget(popup, layout_menu[0]);
}

fn render_audio_view(frame: &mut Frame, _state: &mut State) {
    let layout_horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .flex(Flex::Center)
        .constraints(vec![Constraint::Length(100)])
        .split(frame.area());

    let layout_vertical = Layout::default()
        .direction(Direction::Vertical)
        .flex(Flex::Center)
        .constraints(vec![Constraint::Length(10)])
        .split(layout_horizontal[0]);

    let split_layout = Layout::default()
        .direction(Direction::Horizontal)
        .flex(Flex::Center)
        .constraints(vec![Constraint::Fill(1), Constraint::Fill(1)])
        .split(layout_vertical[0]);

    let home_dir = env::home_dir().unwrap();
    let full_path = home_dir.join(".local/share/grainiac/");
    let paths = fs::read_dir(full_path).unwrap();
    let mut spans: Vec<Span> = vec![];
    for path in paths {
        spans.push(Span::from(format!(
            "{}",
            path.unwrap().file_name().display()
        )));
    }

    let list = List::new(spans.clone())
        .block(Block::bordered().title("Audios"))
        //.style(Style::new().white())
        .highlight_style(Style::new().italic())
        .highlight_symbol(">>")
        .repeat_highlight_symbol(true)
        .direction(ListDirection::TopToBottom);

    let preview = List::new(spans)
        .block(Block::bordered().title("TBD"))
        //.style(Style::new().white())
        .highlight_style(Style::new().italic())
        .highlight_symbol(">>")
        .repeat_highlight_symbol(true)
        .direction(ListDirection::TopToBottom);

    frame.render_widget(list, split_layout[0]);
    frame.render_widget(preview, split_layout[1]);
}
