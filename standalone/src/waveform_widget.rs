use brailles::{NUM_STATES, STATES, STATE_10};
use grainiac_core::{voice::PlayDirection, DrawData};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Text},
    widgets::{Paragraph, Widget},
};

mod brailles;

pub struct Waveform {
    label: String,
    draw_data: DrawData,
}

impl Waveform {
    pub fn from(label: &str, draw_data: DrawData) -> Self {
        Self {
            label: label.to_string(),
            draw_data,
        }
    }
}

impl Widget for Waveform {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(1), Constraint::Length(6)])
            .split(area);

        // draw waveform
        for (x, sample) in self.draw_data.buffer.iter().enumerate() {
            let state = STATES[(sample * (NUM_STATES) as f32).clamp(0.0, 10.0) as usize];

            for (index, char) in state.iter().enumerate() {
                let char_str = char.to_string();
                buf[(x as u16 + layout[1].left(), layout[1].top() + index as u16)]
                    .set_symbol(char_str.as_str());
            }
        }

        // draw grains
        self.draw_data.voice_data.iter().for_each(|data| {
            let y = (data.2 + 1.0) / 2.0;
            let x = (data.0 * self.draw_data.buffer.len() as f32) as u16 + layout[1].left();
            buf[(
                x,
                (layout[1].top() + (y * layout[1].height as f32) as u16) - 1,
            )]
                .set_symbol("O")
                .set_style(Style::default().fg(Color::LightYellow));
        });

        // draw loop length
        for (index, char) in STATE_10.iter().enumerate() {
            let char_str = char.to_string();
            let loop_start = self.draw_data.loop_area.0 * self.draw_data.buffer.len() as f32;
            let loop_length = (self.draw_data.loop_area.1 + self.draw_data.loop_area.0)
                * self.draw_data.buffer.len() as f32;
            buf[(
                loop_start as u16 + layout[1].left(),
                layout[1].top() + index as u16,
            )]
                .set_symbol(char_str.as_str())
                .set_style(Style::default().fg(Color::Rgb(200, 100, 100)));

            buf[(
                (loop_length.clamp(loop_start + 1.0, self.draw_data.buffer.len() as f32) as u16
                    + layout[1].left()),
                layout[1].top() + index as u16,
            )]
                .set_symbol(char_str.as_str())
                .set_style(Style::default().fg(Color::Rgb(100, 200, 100)));
        }

        // draw infos
        let pitch = format!("{:.2}", (12.0 * self.draw_data.pitch.log2()));
        let play_speed = format!("{:.2}", self.draw_data.play_speed);
        let is_hold = if self.draw_data.is_hold { "[X]" } else { "[ ]" };
        let play_dir = match self.draw_data.play_dir {
            PlayDirection::Forward => ">>",
            PlayDirection::Backward => "<<",
        };
        let grain_dir = match self.draw_data.grain_dir {
            PlayDirection::Forward => ">>",
            PlayDirection::Backward => "<<",
        };
        let text = Text::from(Line::from(vec![
            self.label.into(),
            "   ".into(),
            "Hold: ".into(),
            is_hold.to_string().blue().bold(),
            " | ".into(),
            "Play Dir: ".into(),
            play_dir.to_string().blue().bold(),
            " | ".into(),
            "Grain Dir: ".into(),
            grain_dir.to_string().blue().bold(),
            " | ".into(),
            "Pitch: ".into(),
            pitch.to_string().blue().bold(),
            " | ".into(),
            "Speed: ".into(),
            play_speed.to_string().blue().bold(),
        ]));
        Paragraph::new(text).render(layout[0], buf);
    }
}
