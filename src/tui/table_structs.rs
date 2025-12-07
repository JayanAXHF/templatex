use ratatui::{style::Stylize, text::Span, widgets::Widget};

use crate::templating::LoadedTemplateDir;

#[derive(Debug, Default)]
pub struct DataSlice<'a>(pub &'a [LoadedTemplateDir]);

impl<'a> rat_ftable::TableData<'a> for DataSlice<'a> {
    fn rows(&self) -> usize {
        self.0.len()
    }
    fn render_cell(
        &self,
        _ctx: &rat_ftable::TableContext,
        column: usize,
        row: usize,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        if let Some(d) = self.0.get(row) {
            match column {
                0 => {
                    let span = Span::from(d.dir.display().to_string());
                    span.render(area, buf);
                }
                1 => {
                    let span = Span::from(d.name()).blue();
                    span.render(area, buf);
                }
                2 => {
                    if let Some(desc) = d.description() {
                        let span = Span::from(desc);
                        span.render(area, buf);
                    }
                }
                _ => {}
            }
        }
    }
}
