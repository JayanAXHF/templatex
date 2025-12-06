use super::{AppEvent, Global};
use crate::filter::Filter;
use crate::filter::FilterFn;
use crate::templating::LoadedTemplateDir;
use crate::tui::table_structs::DataSlice;
use color_eyre::eyre::Report as Error;
use rat_ftable::Table;
use rat_ftable::TableState;
use rat_ftable::event::ct_event;
use rat_ftable::selection::RowSelection;
use rat_ftable::textdata::Cell;
use rat_ftable::textdata::Row;
use rat_salsa::{Control, SalsaContext};
use rat_theme4::WidgetStyle;
use rat_widget::event::TextOutcome;
use rat_widget::event::{HandleEvent, Regular, try_flow};
use rat_widget::focus::impl_has_focus;

use rat_widget::text::TextStyle;
use rat_widget::text_input::TextInput;
use rat_widget::text_input::TextInputState;
use ratatui::buffer::Buffer;
use ratatui::crossterm;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::Style;
use ratatui::style::Stylize;
use ratatui::text::Span;
use ratatui::widgets::StatefulWidget;
use ratatui::widgets::Widget;

#[derive(Debug, Default)]
pub struct Minimal {
    pub dirs: Vec<LoadedTemplateDir>,
    pub input: TextInputState,
    pub table: TableState<RowSelection>,
    pub filter: Filter<String>,
}

impl Minimal {
    pub fn get_dirs(&self) -> Vec<LoadedTemplateDir> {
        let f = self.filter.clone();
        if f.get_filter().is_empty() {
            return self.dirs.clone();
        }
        self.dirs
            .iter()
            .filter(|d| f.filter(d.name.clone()))
            .cloned()
            .collect::<Vec<_>>()
    }
}

pub fn render(
    area: Rect,
    buf: &mut Buffer,
    state: &mut Minimal,
    ctx: &mut Global,
) -> Result<(), Error> {
    let input = TextInput::new().styles(ctx.theme.style(WidgetStyle::TEXTAREA));

    let &[input_area_all, table_area] = Layout::new(
        Direction::Vertical,
        vec![Constraint::Length(1), Constraint::Fill(1)],
    )
    .split(area)
    .as_ref() else {
        unreachable!()
    };
    let &[prefix_area, input_area] = Layout::new(
        Direction::Horizontal,
        vec![Constraint::Length(2), Constraint::Fill(1)],
    )
    .split(input_area_all)
    .as_ref() else {
        unreachable!()
    };
    let bg: TextStyle = ctx.theme.style(WidgetStyle::TEXTAREA);
    let bg = bg.style.bg.unwrap_or_default();

    let prefix_text = if let Some(f) = ctx.focus().focused()
        && f == state.input.focus
    {
        ">"
    } else {
        "v"
    };

    buf.set_style(prefix_area, Style::default().bg(bg));

    let prefix = Span::styled(
        prefix_text,
        if let Some(f) = ctx.focus().focused()
            && f == state.input.focus
        {
            Style::default().cyan()
        } else {
            Style::default()
        },
    );
    prefix.render(prefix_area, buf);
    input.render(input_area, buf, &mut state.input);
    let data = DataSlice(&state.get_dirs());
    let table = Table::<RowSelection>::new()
        .data(data)
        .column_spacing(1)
        .header(Row::new([
            Cell::from("Dir"),
            Cell::from("Name"),
            Cell::from("Desc"),
        ]))
        .widths([
            Constraint::Percentage(30),
            Constraint::Percentage(20),
            Constraint::Fill(1),
        ])
        .styles(ctx.theme.style(WidgetStyle::TABLE));
    table.render(table_area, buf, &mut state.table);

    Ok(())
}

impl_has_focus!(table, input for Minimal);

pub fn init(_state: &mut Minimal, ctx: &mut Global) -> Result<(), Error> {
    ctx.focus().first();
    Ok(())
}

#[allow(unused_variables)]
pub fn event(
    event: &AppEvent,
    state: &mut Minimal,
    ctx: &mut Global,
) -> Result<Control<AppEvent>, Error> {
    match event {
        AppEvent::Event(event) => {
            try_flow!(match state.input.handle(event, Regular) {
                TextOutcome::TextChanged => {
                    let s = state.input.value();
                    state.filter.replace_filter(vec![s]);
                    Control::Changed
                }
                v => v.into(),
            });
            try_flow!(state.table.handle(event, Regular));
            try_flow!(match event {
                ct_event!(keycode press Enter) => Control::Quit,
                _ => Control::Continue,
            });

            Ok(Control::Continue)
        }
        _ => Ok(Control::Continue),
    }
}
