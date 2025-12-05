use super::{AppEvent, Global};
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
use rat_widget::event::{HandleEvent, Regular, try_flow};
use rat_widget::focus::impl_has_focus;

use ratatui::buffer::Buffer;
use ratatui::crossterm;
use ratatui::layout::{Constraint, Rect};
use ratatui::widgets::StatefulWidget;

#[derive(Debug, Default)]
pub struct Minimal {
    pub dirs: Vec<LoadedTemplateDir>,
    pub table: TableState<RowSelection>,
}

pub fn render(
    area: Rect,
    buf: &mut Buffer,
    state: &mut Minimal,
    ctx: &mut Global,
) -> Result<(), Error> {
    let data = DataSlice(&state.dirs);
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
    table.render(area, buf, &mut state.table);

    Ok(())
}

impl_has_focus!(table for Minimal);

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
