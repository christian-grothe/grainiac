use ratatui::{
    text::{Line, Span},
    widgets::Widget,
};

use crate::widgets::fader_widget::brailles::{BRAILLE_FULL, MAX_CHAR, STATES, STEPS_PER_CHAR};
mod brailles;

pub struct Fader {
    label: String,
    value: f32,
}

impl Fader {
    pub fn new(label: &str, value: f32) -> Self {
        Self {
            label: label.to_string(),
            value,
        }
    }
}

impl Widget for Fader {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let full_brailles = MAX_CHAR as f32 * self.value;
        let left_over = full_brailles - full_brailles.floor();
        let last_braille = STATES[(left_over * STEPS_PER_CHAR as f32) as usize];

        let mut brailles = vec![BRAILLE_FULL; full_brailles.floor() as usize];
        brailles.push(last_braille);

        let brailles_span = Span::from(brailles.iter().collect::<String>());
        let label_span = Span::from(format!("{}: ", self.label));

        Line::from(vec![label_span, brailles_span]).render(area, buf);
    }
}
