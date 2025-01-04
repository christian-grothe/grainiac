use brailles::{NUM_STATES, STATES, STATE_10};
use grainiac_core::DrawData;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Widget},
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
        let block = Block::default()
            .title(Span::styled(self.label, Style::default().fg(Color::Green)))
            .borders(Borders::LEFT);

        block.clone().render(area, buf);

        let inner_area = block.inner(area);

        // draw waveform
        for (x, sample) in self.draw_data.buffer.iter().enumerate() {
            let state = STATES[(sample * NUM_STATES as f32) as usize];

            for (index, char) in state.iter().enumerate() {
                let char_str = char.to_string();
                buf[(
                    x as u16 + inner_area.left(),
                    inner_area.top() + index as u16,
                )]
                    .set_symbol(char_str.as_str());
            }
        }

        // draw loop length
        for (index, char) in STATE_10.iter().enumerate() {
            let char_str = char.to_string();
            let loop_start = self.draw_data.loop_area.0 * self.draw_data.buffer.len() as f32;
            let loop_length = (self.draw_data.loop_area.1 + self.draw_data.loop_area.0)
                * self.draw_data.buffer.len() as f32;
            buf[(
                loop_start as u16 + inner_area.left(),
                inner_area.top() + index as u16,
            )]
                .set_symbol(char_str.as_str())
                .set_style(Style::default().fg(Color::Green));

            buf[(
                (loop_length.clamp(0.0, self.draw_data.buffer.len() as f32) as u16
                    + inner_area.left()),
                inner_area.top() + index as u16,
            )]
                .set_symbol(char_str.as_str())
                .set_style(Style::default().fg(Color::Red));
        }
    }
}
