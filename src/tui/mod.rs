pub mod table_structs;
pub mod ui;
///
/// Inline rendering in the console.
/// Uses only 10 lines for the UI.
///
use self::ui::Minimal;
use crate::config::Theme;
use crate::templating::LoadedTemplateDir;
use color_eyre::eyre::Report as Error;
use rat_salsa::event::RenderedEvent;
use rat_salsa::poll::{PollCrossterm, PollRendered};
use rat_salsa::terminal::CrosstermTerminal;
use rat_salsa::timer::TimeOut;
use rat_salsa::{Control, RunConfig, SalsaAppContext, SalsaContext, run_tui};
use rat_theme4::create_salsa_theme;
use rat_theme4::theme::SalsaTheme;
use rat_widget::event::{ConsumedEvent, Dialog, HandleEvent, Regular, ct_event};
use rat_widget::focus::FocusBuilder;
use rat_widget::msgdialog::{MsgDialog, MsgDialogState};
use rat_widget::statusline::{StatusLine, StatusLineState};
use ratatui::buffer::Buffer;
use ratatui::crossterm::terminal::disable_raw_mode;
use ratatui::crossterm::{self, execute};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::StatefulWidget;
use std::io::stdout;
use std::process;
use std::time::{Duration, SystemTime};

pub fn picker(
    dirs: Vec<LoadedTemplateDir>,
    user_theme: Option<Theme>,
) -> Result<LoadedTemplateDir, Error> {
    let config = Config::default();
    let theme = match user_theme {
        Some(t) => t.into(),
        None => create_salsa_theme("Imperial Shell"),
    };
    let mut global = Global::new(config, theme);
    let ui = Minimal {
        dirs: dirs.clone(),
        ..Default::default()
    };
    let mut state = Scenery {
        ui,
        ..Default::default()
    };

    run_tui(
        init,
        render,
        event,
        error,
        &mut global,
        &mut state,
        RunConfig::new(CrosstermTerminal::inline(12, true)?)
            .poll(PollCrossterm)
            .poll(PollRendered),
    )?;
    let dirs = state.ui.get_dirs();
    let template = state
        .ui
        .table
        .selected_checked()
        .ok_or(Error::msg("No template selected"))?;
    let template = dirs[template].clone();

    Ok(template)
}

/// Globally accessible data/state.
pub struct Global {
    ctx: SalsaAppContext<AppEvent, Error>,
    pub cfg: Config,
    pub theme: SalsaTheme,
}

impl SalsaContext<AppEvent, Error> for Global {
    fn set_salsa_ctx(&mut self, app_ctx: SalsaAppContext<AppEvent, Error>) {
        self.ctx = app_ctx;
    }

    #[inline(always)]
    fn salsa_ctx(&self) -> &SalsaAppContext<AppEvent, Error> {
        &self.ctx
    }
}

impl Global {
    pub fn new(cfg: Config, theme: SalsaTheme) -> Self {
        Self {
            ctx: Default::default(),
            cfg,
            theme,
        }
    }
}

/// Configuration.
#[derive(Debug, Default)]
pub struct Config {}

/// Application wide messages.

#[derive(Debug)]
pub enum AppEvent {
    Timer(TimeOut),
    Event(crossterm::event::Event),
    Rendered,
    Message(String),
    Status(usize, String),
}

impl From<RenderedEvent> for AppEvent {
    fn from(_: RenderedEvent) -> Self {
        Self::Rendered
    }
}

impl From<TimeOut> for AppEvent {
    fn from(value: TimeOut) -> Self {
        Self::Timer(value)
    }
}

impl From<crossterm::event::Event> for AppEvent {
    fn from(value: crossterm::event::Event) -> Self {
        Self::Event(value)
    }
}

#[derive(Debug, Default)]
pub struct Scenery {
    pub ui: Minimal,
    pub status: StatusLineState,
    pub error_dlg: MsgDialogState,
}

pub fn render(
    area: Rect,
    buf: &mut Buffer,
    state: &mut Scenery,
    ctx: &mut Global,
) -> Result<(), Error> {
    let t0 = SystemTime::now();

    // forward
    ui::render(area, buf, &mut state.ui, ctx)?;

    let layout = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1), //
    ])
    .split(area);

    if state.error_dlg.active() {
        MsgDialog::new().render(layout[1], buf, &mut state.error_dlg);
    }

    let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
    state.status.status(1, format!("R {:.0?}", el).to_string());

    let status_layout = Layout::horizontal([
        Constraint::Fill(61), //
        Constraint::Fill(39),
    ])
    .split(layout[0]);

    StatusLine::new()
        .layout([
            Constraint::Fill(1),
            Constraint::Length(8),
            Constraint::Length(8),
        ])
        .render(status_layout[1], buf, &mut state.status);

    Ok(())
}

pub fn init(state: &mut Scenery, ctx: &mut Global) -> Result<(), Error> {
    ctx.set_focus(FocusBuilder::build_for(&state.ui));
    ui::init(&mut state.ui, ctx)?;
    Ok(())
}

pub fn event(
    event: &AppEvent,
    state: &mut Scenery,
    ctx: &mut Global,
) -> Result<Control<AppEvent>, Error> {
    let t0 = SystemTime::now();

    let mut r = match event {
        AppEvent::Event(event) => {
            let mut r = match &event {
                ct_event!(resized) => Control::Changed,
                ct_event!(key press CONTROL-'q') => Control::Quit,
                ct_event!(key press CONTROL-'c') => {
                    disable_raw_mode()?;
                    execute!(stdout(), crossterm::cursor::Show)?;
                    process::exit(0);
                }
                _ => Control::Continue,
            };

            r = r.or_else(|| {
                if state.error_dlg.active() {
                    state.error_dlg.handle(event, Dialog).into()
                } else {
                    Control::Continue
                }
            });

            let f = ctx.focus_mut().handle(event, Regular);
            ctx.queue(f);

            r
        }
        AppEvent::Rendered => {
            ctx.set_focus(FocusBuilder::rebuild_for(&state.ui, ctx.take_focus()));
            Control::Continue
        }
        AppEvent::Message(s) => {
            state.error_dlg.append(s.as_str());
            Control::Changed
        }
        AppEvent::Status(n, s) => {
            state.status.status(*n, s);
            Control::Changed
        }
        _ => Control::Continue,
    };

    r = r.or_else_try(|| ui::event(event, &mut state.ui, ctx))?;

    let el = t0.elapsed()?;
    state.status.status(2, format!("E {:.0?}", el).to_string());

    Ok(r)
}

pub fn error(
    event: Error,
    state: &mut Scenery,
    _ctx: &mut Global,
) -> Result<Control<AppEvent>, Error> {
    state.error_dlg.append(format!("{:?}", &*event).as_str());
    Ok(Control::Changed)
}
