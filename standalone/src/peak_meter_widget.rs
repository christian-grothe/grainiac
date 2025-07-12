use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::Widget,
};

pub struct PeakMeter {
    label: String,
    peak: f32,
    max_length: usize,
}

impl PeakMeter {
    pub fn from(label: &str, peak: f32, max_length: usize) -> Self {
        Self {
            label: label.to_string(),
            peak,
            max_length,
        }
    }
}

impl Widget for PeakMeter {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let peak = (self.peak * self.max_length as f32) as usize;

        let bar_style = if peak > 12 {
            Style::new().red()
        } else if peak > 8 {
            Style::new().yellow()
        } else {
            Style::new().white()
        };

        let bar = Span::styled(">".repeat(peak), bar_style);
        let label = Span::from(format!("{} ", self.label));
        let line = Line::from(vec![label, bar]);

        line.render(area, buf);
    }
}
