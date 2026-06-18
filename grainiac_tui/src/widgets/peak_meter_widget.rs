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
}

impl PeakMeter {
    pub fn from(label: &str, peak: f32) -> Self {
        Self {
            label: label.to_string(),
            peak,
        }
    }
}

const STEPS_PER_CELL: usize = 8;
const BLOCKS: [char; STEPS_PER_CELL] = ['▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];

impl Widget for PeakMeter {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let label = format!("{} ", self.label);
        let label_width = label.chars().count() as u16;
        let bar_width = area.width.saturating_sub(label_width);

        let peak = self.peak.clamp(0.0, 1.0);
        let total_steps = (peak * bar_width as f32 * STEPS_PER_CELL as f32) as usize;
        let full_cells = total_steps / STEPS_PER_CELL;
        let partial = total_steps % STEPS_PER_CELL;

        let bar_style = if peak > 0.8 {
            Style::new().red()
        } else if peak > 0.55 {
            Style::new().yellow()
        } else {
            Style::new().white()
        };

        let mut bar = String::with_capacity(bar_width as usize);
        for _ in 0..full_cells {
            bar.push('█');
        }
        if full_cells < bar_width as usize && partial > 0 {
            bar.push(BLOCKS[partial - 1]);
        }

        let label_span = Span::from(label);
        let bar_span = Span::styled(bar, bar_style);

        Line::from(vec![label_span, bar_span]).render(area, buf);
    }
}
