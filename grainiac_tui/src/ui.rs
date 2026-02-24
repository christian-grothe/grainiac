use std::{env, fs};

use ratatui::{
    layout::{Constraint, Direction, Flex, Layout},
    style::{Style, Stylize},
    text::Span,
    widgets::{Block, List, ListDirection, ListItem},
    Frame,
};

use crate::{
    INSTANCE_NUM, state::{NumMode, State, View}, widgets::{peak_meter_widget::PeakMeter, track_widget::Track}
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

    let mut constraints = vec![Constraint::Length(1), Constraint::Length(1)];

    for _ in 0..INSTANCE_NUM {
        constraints.push(Constraint::Length(11));
    }

    constraints.push(Constraint::Length(1));

    let layout_vertical = Layout::default()
        .direction(Direction::Vertical)
        .flex(Flex::Center)
        .constraints(constraints)
        .split(layout_horizontal[0]);

    for i in 0..INSTANCE_NUM {
        let track = Track::from(&(i + 1).to_string(), out_buf[i].clone());
        frame.render_widget(track, layout_vertical[i + 2]);
    }

    let span = match state.num_mode {
        NumMode::LoadPreset => Span::from("Press 0 - 9 to LOAD a PRESET"),
        NumMode::SavePreset => Span::from("Press 0 - 9 to SAVE a PRESET"),
        NumMode::LoadAudio => Span::from("Press 0 - 9 to LOAD an AUDIO"),
        NumMode::SaveAudio => Span::from("Press 0 - 9 to SAVE an AUDIO"),
    };

    frame.render_widget(span, layout_vertical[layout_vertical.len() - 1]);

    let peak_meter_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Fill(1), Constraint::Fill(1)])
        .split(layout_vertical[0]);

    let peak_meter_input = PeakMeter::from("Input", out_buf[0].input_peak, 15);
    let peak_meter_output = PeakMeter::from("Output", out_buf[0].output_peak, 15);
    frame.render_widget(peak_meter_input, peak_meter_layout[0]);
    frame.render_widget(peak_meter_output, peak_meter_layout[1]);
}

fn render_preset_view(frame: &mut Frame, state: &mut State) {
    let layout_horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .flex(Flex::Center)
        .constraints(vec![Constraint::Length(100)])
        .split(frame.area());

    let layout_vertical = Layout::default()
        .direction(Direction::Vertical)
        .flex(Flex::Center)
        .constraints(vec![Constraint::Length(16)])
        .split(layout_horizontal[0]);

    let split_layout = Layout::default()
        .direction(Direction::Horizontal)
        .flex(Flex::Center)
        .constraints(vec![Constraint::Fill(1), Constraint::Fill(1)])
        .split(layout_vertical[0]);

    let list_items: Vec<ListItem> = state
        .presets
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let content = if state.selected_preset_idx == i {
                format!(">> {}", p.name)
            } else {
                format!("   {}", p.name)
            };
            ListItem::new(content)
        })
        .collect();

    let list = List::new(list_items)
        .block(Block::bordered().title("Presets"))
        .direction(ListDirection::TopToBottom);

    let selected_preset = &state.presets[state.selected_preset_idx];
    let preview = List::new(selected_preset.to_preview())
        .block(Block::bordered())
        .direction(ListDirection::TopToBottom);

    frame.render_widget(list, split_layout[0]);
    frame.render_widget(preview, split_layout[1]);
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
        .constraints(vec![Constraint::Length(12)])
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
