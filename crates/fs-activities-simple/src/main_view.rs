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
use fs_components_widgets::{
    CalendarItem, CalendarNames, CalendarView, Element, ViewRange, styles,
};

use crate::preview::{Mode, Program, mode, newest_day, oldest_day, rows};

/// Width of the column of filters.
const LEFT: f32 = 210.0;
const PAD: f32 = 16.0;
const GAP: f32 = 12.0;
/// What the marker above the timeline needs, and the search field; the agenda gets the rest.
const NOTICE_HEIGHT: f32 = 100.0;
const SEARCH_HEIGHT: f32 = 40.0;

/// What the person has chosen in the filters and typed in the search.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Filter {
    /// Only this program, or all.
    pub program: Option<Program>,
    /// Words that must be in the text (any case); empty means no search.
    pub query: String,
}

impl Filter {
    /// Whether nothing is filtered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.program.is_none() && self.query.trim().is_empty()
    }
}

/// The marker at the top of the view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Notice {
    /// "Preview": the rows are example data; the button hides them.
    Shown,
    /// "Preview": the example is hidden (the empty state shows); the button shows it again.
    Hidden,
}

/// The marker for this state: there is one exactly while the example data are the data (mode [`Mode::Preview`]).
#[must_use]
pub fn notice(mode: Mode, example: bool) -> Option<Notice> {
    match (mode, example) {
        (Mode::Live, _) => None,
        (Mode::Preview, true) => Some(Notice::Shown),
        (Mode::Preview, false) => Some(Notice::Hidden),
    }
}

/// The rows to draw: the example while in preview and not hidden, never in live mode (the client cannot read the server's
/// activities yet, so live shows none); narrowed by the program and by the words of the search.
#[must_use]
pub fn visible(
    mode: Mode,
    example: bool,
    all: &[(Program, CalendarItem)],
    filter: &Filter,
) -> Vec<CalendarItem> {
    if mode != Mode::Preview || !example {
        return Vec::new();
    }
    let query = filter.query.trim().to_lowercase();
    all.iter()
        .filter(|(program, _)| filter.program.is_none_or(|p| p == *program))
        .filter(|(_, item)| query.is_empty() || item.title.to_lowercase().contains(&query))
        .map(|(_, item)| item.clone())
        .collect()
}

/// The words of the view, read once.
struct Labels {
    preview_title: String,
    preview_text: String,
    preview_hidden: String,
    hide: String,
    show: String,
    filter: String,
    all: String,
    programs: Vec<(Program, String)>,
    mine: String,
    others: String,
    why: String,
    search: String,
    empty_title: String,
    empty_text: String,
    nothing_title: String,
    nothing_text: String,
    reset: String,
    names: Arc<CalendarNames>,
}

impl Labels {
    fn read(texts: &Mf2Texts) -> Self {
        let names = CalendarNames::new(
            std::array::from_fn(|i| texts.plain(&format!("weekday-{}", i + 1))),
            std::array::from_fn(|i| texts.plain(&format!("month-{}", i + 1))),
            texts.plain("agenda-all-day"),
            texts.plain("agenda-empty"),
        );
        Self {
            preview_title: texts.plain("preview-title"),
            preview_text: texts.plain("preview-text"),
            preview_hidden: texts.plain("preview-hidden-text"),
            hide: texts.plain("preview-hide"),
            show: texts.plain("preview-show"),
            filter: texts.plain("filter-title"),
            all: texts.plain("filter-all"),
            programs: Program::ALL
                .iter()
                .map(|p| (*p, texts.plain(p.word())))
                .collect(),
            mine: texts.plain("filter-mine"),
            others: texts.plain("filter-others"),
            why: texts.plain("who-missing"),
            search: texts.plain("search-placeholder"),
            empty_title: texts.plain("empty-title"),
            empty_text: texts.plain("empty-text"),
            nothing_title: texts.plain("nothing-title"),
            nothing_text: texts.plain("nothing-text"),
            reset: texts.plain("nothing-reset"),
            names: Arc::new(names),
        }
    }
}

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
            .gap(2.0)
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
            .gap(6.0)
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
    fn body(&self, filter: &Filter, items: Vec<CalendarItem>, size: (f32, f32)) -> Div {
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
        CalendarView::new(
            ViewRange::Agenda,
            oldest_day(),
            Arc::clone(&self.labels.names),
            Arc::new(items),
        )
        .today(newest_day())
        .named("activities")
        .build(size.0, size.1)
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
        let mut page = div().flex_col().gap(GAP).p(PAD).w(width).h(height);
        if let Some(kind) = marker {
            page = page.child(self.marker(kind));
        }
        let timeline = div()
            .flex_col()
            .gap(GAP)
            .w(right_width)
            .child(self.search(&filter))
            .child(self.body(&filter, items, body_size));
        Box::new(
            page.child(
                div()
                    .flex_row()
                    .gap(GAP)
                    .child(self.filters(&filter))
                    .child(timeline),
            ),
        )
    }

    fn styles(&self) -> String {
        styles()
    }
}

#[cfg(test)]
mod tests {
    use super::{Filter, Main, Notice, notice, visible};
    use crate::preview::{Mode, Program};
    use crate::words::words;
    use fs_components_words::Locale;

    fn main(caps: &[&str], example: bool) -> Main {
        let caps: Vec<String> = caps.iter().map(|s| (*s).to_owned()).collect();
        Main::new(&words(&Locale::english()), &caps, example)
    }

    #[test]
    fn the_marker_exists_exactly_in_preview_mode() {
        assert_eq!(notice(Mode::Preview, true), Some(Notice::Shown));
        assert_eq!(notice(Mode::Preview, false), Some(Notice::Hidden));
        assert_eq!(notice(Mode::Live, true), None);
        assert_eq!(notice(Mode::Live, false), None);
    }

    #[test]
    fn the_example_is_there_without_the_capability_and_gone_with_it() {
        let preview = main(&[], true);
        assert_eq!(preview.mode(), Mode::Preview);
        assert_eq!(preview.items().len(), 12);
        assert_eq!(preview.notice(), Some(Notice::Shown));
        let live = main(&["history"], true);
        assert_eq!(live.mode(), Mode::Live);
        assert!(live.items().is_empty(), "no example once the server delivers");
        assert_eq!(live.notice(), None);
    }

    #[test]
    fn hiding_the_example_shows_the_empty_state_and_the_marker_stays() {
        let view = main(&[], true);
        view.set_example(false);
        assert!(view.items().is_empty());
        assert_eq!(view.notice(), Some(Notice::Hidden));
        view.set_example(true);
        assert_eq!(view.items().len(), 12);
    }

    #[test]
    fn the_filter_narrows_by_program_and_by_words() {
        let view = main(&[], true);
        view.set_filter(Filter {
            program: Some(Program::Texts),
            query: String::new(),
        });
        let texts = view.items();
        assert_eq!(texts.len(), 4);
        assert!(texts.iter().all(|i| i.marks.iter().any(|m| m == "Texts")));
        view.set_filter(Filter {
            program: None,
            query: "  FOLDER ".into(),
        });
        assert_eq!(view.items().len(), 1);
        view.set_filter(Filter {
            program: Some(Program::Tasks),
            query: "folder".into(),
        });
        assert!(view.items().is_empty());
    }

    #[test]
    fn nothing_is_filtered_by_default() {
        assert!(Filter::default().is_empty());
        assert!(
            visible(Mode::Live, true, &[], &Filter::default()).is_empty()
        );
    }
}
