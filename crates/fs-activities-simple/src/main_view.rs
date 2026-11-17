//! The main view: the timeline of activities in the look of its models (Nextcloud Activity, "My Activity"): filters on the
//! left, the search above, the days as an agenda in the middle.
//!
//! Everything on the screen is a block of fs-components: the agenda of the `calendar-view`, the `search-field`, the buttons,
//! the `alert` (the "Preview" marker) and the `empty-state`. While the server offers no capability `history`, the rows are
//! the example of `preview.rs` and the view says so at the top; after that the view shows what the server delivers (the
//! client cannot read activities yet, so the empty state says that plainly). The decisions ([`notice`], [`visible`]) are
//! pure functions, so a test can read them.
//!
//! Pattern: Adapter (our rows in, blocks of the shell out) + State (the filter and the example switch live outside the
//! element, because the surface is built again at every change).

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use fs_components::blinc::cn::components::button::{ButtonBuilder, ButtonSize, ButtonVariant};
use fs_components::blinc::core::BlincContextState;
use fs_components::blinc::layout::div::{Div, div};
use fs_components::blinc::layout::prelude::text;
use fs_components_app::Surface;
use fs_components_mf2::Mf2Texts;
use fs_components_primitives::SearchField;
use fs_components_widgets::states::{Alert, EmptyState, Severity};
use fs_components_widgets::{CalendarItem, Element, styles};

use crate::filter::{Filter, Notice, notice, visible};
use crate::labels::Labels;
use crate::preview::{Mode, Program, mode, rows};
use crate::timeline::agenda;

/// Width of the column of filters.
const LEFT: f32 = 210.0;
/// Padding of the page in pixels (Blinc counts padding in units of 4 px).
const PAD: f32 = 16.0;
/// Gap between blocks in pixels (Blinc counts gaps in units of 4 px).
const GAP: f32 = 12.0;
/// What the marker above the timeline needs, and the search field; the agenda gets the rest.
const NOTICE_HEIGHT: f32 = 100.0;
const SEARCH_HEIGHT: f32 = 40.0;

#[derive(Clone, Default)]
struct State {
    filter: Filter,
    /// Whether the example is shown (preview mode only).
    example: bool,
}

fn locked(state: &Mutex<State>) -> MutexGuard<'_, State> {
    state.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Ask the window to draw the surface again (a surface is built again at every change; nothing to do outside a window).
fn redraw() {
    if BlincContextState::is_initialized() {
        BlincContextState::get().request_rebuild();
    }
}

/// The main screen.
pub struct Main {
    labels: Labels,
    rows: Vec<(Program, CalendarItem)>,
    mode: Mode,
    state: Arc<Mutex<State>>,
}

impl Main {
    /// The main view for a server that offers `capabilities`; `example` says whether the example is shown at first (it only
    /// matters in preview mode).
    #[must_use]
    pub fn new(texts: &Mf2Texts, capabilities: &[String], example: bool) -> Self {
        Self {
            labels: Labels::read(texts),
            rows: rows(texts),
            mode: mode(capabilities),
            state: Arc::new(Mutex::new(State {
                filter: Filter::default(),
                example,
            })),
        }
    }

    /// Example or real data.
    #[must_use]
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// The marker shown now.
    #[must_use]
    pub fn notice(&self) -> Option<Notice> {
        notice(self.mode, locked(&self.state).example)
    }

    /// The rows shown now.
    #[must_use]
    pub fn items(&self) -> Vec<CalendarItem> {
        let state = locked(&self.state);
        visible(self.mode, state.example, &self.rows, &state.filter)
    }

    /// Show or hide the example (what the button of the marker does).
    pub fn set_example(&self, shown: bool) {
        locked(&self.state).example = shown;
    }

    /// Narrow the rows (what the filters and the search do).
    pub fn set_filter(&self, filter: Filter) {
        locked(&self.state).filter = filter;
    }

    fn marker(&self, kind: Notice) -> impl Element + use<> {
        let state = Arc::clone(&self.state);
        let (text, label, next) = match kind {
            Notice::Shown => (&self.labels.preview_text, &self.labels.hide, false),
            Notice::Hidden => (&self.labels.preview_hidden, &self.labels.show, true),
        };
        Alert::new(Severity::Info, text)
            .id("preview")
            .title(&self.labels.preview_title)
            .action(label, move || {
                locked(&state).example = next;
                redraw();
            })
            .build()
    }

    fn choice(&self, key: &str, label: &str, program: Option<Program>, chosen: bool) -> Div {
        let state = Arc::clone(&self.state);
        // The chosen filter is told by a mark in the word, not by colour alone.
        let shown = if chosen {
            format!("\u{25CF} {label}")
        } else {
            label.to_owned()
        };
        let key = format!("filter-{key}");
        div().child(
            ButtonBuilder::with_key(key.clone(), shown)
                .variant(if chosen {
                    ButtonVariant::Secondary
                } else {
                    ButtonVariant::Ghost
                })
                .size(ButtonSize::Small)
                .on_click(move |_| {
                    locked(&state).filter.program = program;
                    redraw();
                })
                .build_component()
                .id(&key),
        )
    }

    /// A filter that cannot work yet: locked, and the reason stands next to it (never silently greyed out).
    fn locked_choice(&self, key: &str, label: &str) -> Div {
        let key = format!("filter-{key}");
        div()
            .flex_col()
            .gap(0.5)
            .child(
                div().child(
                    ButtonBuilder::with_key(key.clone(), label.to_owned())
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Small)
                        .disabled(true)
                        .build_component()
                        .id(&key),
                ),
            )
            .child(text(self.labels.why.clone()))
    }

    fn filters(&self, filter: &Filter) -> Div {
        let mut column = div()
            .flex_col()
            .gap(1.5)
            .w(LEFT)
            .flex_shrink_0()
            .child(text(self.labels.filter.clone()))
            .child(self.choice("all", &self.labels.all, None, filter.program.is_none()));
        for (program, label) in &self.labels.programs {
            column = column.child(self.choice(
                program.word(),
                label,
                Some(*program),
                filter.program == Some(*program),
            ));
        }
        column
            .child(self.locked_choice("mine", &self.labels.mine))
            .child(self.locked_choice("others", &self.labels.others))
    }

    fn search(&self, filter: &Filter) -> impl Element + use<> {
        let field = SearchField::new(&self.labels.search);
        if !filter.query.is_empty() {
            field.set_text_at_end(&filter.query);
        }
        let reader = field.clone();
        let state = Arc::clone(&self.state);
        field
            .on_submit(Arc::new(move || {
                locked(&state).filter.query = reader.text();
                redraw();
            }))
            .build()
    }

    /// The agenda, or the empty state that says why there is none.
    fn body(&self, filter: &Filter, items: &[CalendarItem], size: (f32, f32)) -> Div {
        if items.is_empty() {
            let state = Arc::clone(&self.state);
            let example = locked(&self.state).example;
            let nothing_found = self.mode == Mode::Preview && example && !filter.is_empty();
            let empty = if nothing_found {
                EmptyState::new(&self.labels.nothing_title)
                    .id("nothing")
                    .text(&self.labels.nothing_text)
                    .action(&self.labels.reset, move || {
                        locked(&state).filter = Filter::default();
                        redraw();
                    })
            } else {
                EmptyState::new(&self.labels.empty_title)
                    .id("empty")
                    .text(&self.labels.empty_text)
            };
            return div().w(size.0).h(size.1).child(empty.build());
        }
        agenda(&self.labels.names, items, size)
    }
}

impl Surface for Main {
    fn build(&self, width: f32, height: f32) -> Box<dyn Element> {
        let (filter, example) = {
            let state = locked(&self.state);
            (state.filter.clone(), state.example)
        };
        let marker = notice(self.mode, example);
        let items = visible(self.mode, example, &self.rows, &filter);
        let right_width = (width - 2.0 * PAD - LEFT - GAP).max(0.0);
        let used = 2.0 * PAD + SEARCH_HEIGHT + GAP + marker.map_or(0.0, |_| NOTICE_HEIGHT + GAP);
        let body_size = (right_width, (height - used).max(120.0));
        let mut page = div()
            .flex_col()
            .gap(GAP / 4.0)
            .p(PAD / 4.0)
            .w(width)
            .h(height);
        if let Some(kind) = marker {
            page = page.child(self.marker(kind));
        }
        let timeline = div()
            .flex_col()
            .gap(GAP / 4.0)
            .w(right_width)
            .child(self.search(&filter))
            .child(self.body(&filter, &items, body_size));
        Box::new(
            page.child(
                div()
                    .flex_row()
                    .gap(GAP / 4.0)
                    .child(self.filters(&filter))
                    .child(timeline),
            ),
        )
    }

    fn styles(&self) -> String {
        styles()
    }
}
