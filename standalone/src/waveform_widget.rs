use brailles::{NUM_STATES, STATES, STATE_10};
use grainiac_core::{voice::PlayDirection, DrawData};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
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
            buf[(x, (layout[1].top() + (y * 5.0) as u16))]
                .set_symbol("O")
                .set_style(Style::default().fg(Color::Rgb(255, 255, 186)));
        });

        // draw loop length
        for (index, char) in STATE_10.iter().enumerate() {
            let char_str = char.to_string();
            let loop_start = self.draw_data.state.loop_start * self.draw_data.buffer.len() as f32;
            let loop_length = (self.draw_data.state.loop_length + self.draw_data.state.loop_start)
                * self.draw_data.buffer.len() as f32;
            buf[(
                loop_start as u16 + layout[1].left(),
                layout[1].top() + index as u16,
            )]
                .set_symbol(char_str.as_str())
                .set_style(Style::default().fg(Color::Rgb(255, 179, 186)));

            buf[(
                (loop_length.clamp(loop_start + 1.0, self.draw_data.buffer.len() as f32) as u16
                    + layout[1].left()),
                layout[1].top() + index as u16,
            )]
                .set_symbol(char_str.as_str())
                .set_style(Style::default().fg(Color::Rgb(186, 255, 201)));
        }

        // draw infos
        let pitch = format!("{:.2}", self.draw_data.state.pitch);
        let play_speed = format!("{:.2}", self.draw_data.state.play_speed);
        let gain = format!("{:.2}", self.draw_data.state.gain);

        let is_hold = if self.draw_data.state.is_hold {
            "[X]"
        } else {
            "[ ]"
        };
        let play_dir = match self.draw_data.state.play_dir {
            PlayDirection::Forward => ">>",
            PlayDirection::Backward => "<<",
        };
        let grain_dir = match self.draw_data.state.grain_dir {
            PlayDirection::Forward => ">>",
            PlayDirection::Backward => "<<",
        };

        let spans = Text::from(Line::from(vec![
            Span::styled(self.label, Style::default().bold()),
            Span::styled("   ", Style::default().bold()),
            Span::styled("Hold: ", Style::default().bold()),
            Span::styled(
                is_hold,
                Style::default().fg(Color::Rgb(186, 225, 255)).bold(),
            ),
            Span::styled(" | ", Style::default().bold()),
            Span::styled("Play Dir: ", Style::default().bold()),
            Span::styled(
                play_dir,
                Style::default().fg(Color::Rgb(186, 225, 255)).bold(),
            ),
            Span::styled(" | ", Style::default().bold()),
            Span::styled("Grain Dir: ", Style::default().bold()),
            Span::styled(
                grain_dir,
                Style::default().fg(Color::Rgb(186, 225, 255)).bold(),
            ),
            Span::styled(" | ", Style::default().bold()),
            Span::styled("Pitch: ", Style::default().bold()),
            Span::styled(pitch, Style::default().fg(Color::Rgb(186, 225, 255)).bold()),
            Span::styled(" | ", Style::default().bold()),
            Span::styled("Speed: ", Style::default().bold()),
            Span::styled(
                play_speed,
                Style::default().fg(Color::Rgb(186, 225, 255)).bold(),
            ),
            Span::styled(" | ", Style::default().bold()),
            Span::styled("Gain: ", Style::default().bold()),
            Span::styled(gain, Style::default().fg(Color::Rgb(186, 225, 255)).bold()),
        ]));
        Paragraph::new(spans).render(layout[0], buf);
    }
}
