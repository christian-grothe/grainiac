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

pub struct Track {
    label: String,
    draw_data: DrawData,
}

impl Track {
    pub fn from(label: &str, draw_data: DrawData) -> Self {
        Self {
            label: label.to_string(),
            draw_data,
        }
    }
}

impl Widget for Track {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(3), Constraint::Length(6)])
            .split(area);

        let param_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(1); 3])
            .split(layout[0]);

        let param_line_a = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Min(1)])
            .split(param_layout[0]);

        let param_line_b = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Length(16); 5])
            .split(param_layout[1]);

        let param_line_c = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Length(16); 5])
            .split(param_layout[2]);

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
        self.draw_data.grain_data.iter().for_each(|data| {
            if let Some(data) = data {
                let y = (data.2 + 1.0) / 2.0;
                let x = (data.0 * self.draw_data.buffer.len() as f32) as u16 + layout[1].left();
                buf[(x, (layout[1].top() + (y * 5.0) as u16))]
                    .set_symbol("O")
                    .set_style(Style::default().fg(Color::Rgb(255, 255, 186)));
            }
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
        let is_hold = if self.draw_data.state.is_hold {
            "[X]"
        } else {
            "[ ]"
        };

        let is_rec = if self.draw_data.state.is_recording {
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
            // Span::styled("   ", Style::default().bold()),
            Span::styled("  Rec: ", Style::default().bold()),
            Span::styled(
                is_rec,
                Style::default().fg(Color::Rgb(186, 225, 255)).bold(),
            ),
            Span::styled("  | ", Style::default().bold()),
            Span::styled("Hold: ", Style::default().bold()),
            Span::styled(
                is_hold,
                Style::default().fg(Color::Rgb(186, 225, 255)).bold(),
            ),
            Span::styled("  | ", Style::default().bold()),
            Span::styled("Pl-Dir: ", Style::default().bold()),
            Span::styled(
                play_dir,
                Style::default().fg(Color::Rgb(186, 225, 255)).bold(),
            ),
            Span::styled("  | ", Style::default().bold()),
            Span::styled("Gr-Dir: ", Style::default().bold()),
            Span::styled(
                grain_dir,
                Style::default().fg(Color::Rgb(186, 225, 255)).bold(),
            ),
        ]));

        Paragraph::new(spans).render(param_line_a[0], buf);

        if self.draw_data.state.density < 10.0 {
            Span::from(format!("   den: {:.2}  |", self.draw_data.state.density))
                .render(param_line_b[0], buf);
        } else {
            Span::from(format!("   den: {:.2} |", self.draw_data.state.density))
                .render(param_line_b[0], buf);
        }
        Span::from(format!(
            "   len: {:.2}  |",
            self.draw_data.state.grain_length
        ))
        .render(param_line_c[0], buf);

        Span::from(format!("  spd: {:.2}  |", self.draw_data.state.play_speed))
            .render(param_line_b[1], buf);
        Span::from(format!("  spy: {:.2}  |", self.draw_data.state.spray))
            .render(param_line_c[1], buf);

        if self.draw_data.state.pan > 0.0 {
            Span::from(format!("  pan: R{:.2}  |", self.draw_data.state.pan.abs()))
                .render(param_line_b[2], buf);
        } else if self.draw_data.state.pan < 0.0 {
            Span::from(format!("  pan: L{:.2}  |", self.draw_data.state.pan.abs()))
                .render(param_line_b[2], buf);
        } else {
            Span::from(format!("  pan: -C-    |")).render(param_line_b[2], buf);
        }
        Span::from(format!("  spr: {:.2}   |", self.draw_data.state.spread))
            .render(param_line_c[2], buf);

        Span::from(format!("  att: {:.2}  |", self.draw_data.state.attack))
            .render(param_line_b[3], buf);
        Span::from(format!("  rel: {:.2}  |", self.draw_data.state.release))
            .render(param_line_c[3], buf);

        Span::from(format!("  pch: {} ", self.draw_data.state.pitch)).render(param_line_b[4], buf);
        Span::from(format!("  vol: {:.2} ", self.draw_data.state.gain))
            .render(param_line_c[4], buf);
    }
}

// "density": 3,
// "grain_length": 4,
// "play_speed": 5,
// "spray": 6,
// "pan": 7,
// "spread": 8,
// "attack": 9,
// "release": 10,
// "pitch": 11,
// "gain": 12,
