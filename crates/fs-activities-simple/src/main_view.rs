//! The main view: one screen per view of the navigation on the left (All, By me, By others, then one per program), in the
//! look of its model (Nextcloud Activity): the title is the active view, the marker "Preview" stands once at the top, the
//! search below it, then the days of the timeline, newest first.
//!
//! Everything on the screen is a block of fs-components: the text view for the titles, the `search-field`, the `alert` (the
//! "Preview" marker), the `empty-state` and the icons of the icon set. While the server offers no capability `history`, the
//! rows are the example of `preview.rs` and the view says so at the top; after that the view shows what the server delivers
//! (the client cannot read activities yet, so the empty state says that plainly). A view that cannot work yet ("By me",
//! "By others": the source does not say who did it) stays in the navigation, locked, and says why.
//!
//! Pattern: Adapter (our rows in, blocks of the shell out) + State (the example switch and the search live outside the
//! element and are shared by all screens, because a surface is built again at every change).

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use fs_components::blinc::core::BlincContextState;
use fs_components::blinc::layout::div::div;
use fs_components_app::Surface;
use fs_components_mf2::Mf2Texts;
use fs_components_primitives::SearchField;
use fs_components_widgets::states::{Alert, EmptyState, Severity};
use fs_components_widgets::{Block, Element, Glyph, Inline, TextView, styles};

use crate::filter::{Notice, View, notice, visible};
use crate::labels::Labels;
use crate::preview::{Mode, Row, mode, now_minutes, rows};
use crate::timeline::agenda;

/// Padding of the page in pixels (Blinc counts padding in units of 4 px).
const PAD: f32 = 16.0;
/// Gap between blocks in pixels (Blinc counts gaps in units of 4 px).
const GAP: f32 = 12.0;
/// What the title, the marker and the search need; the timeline gets the rest.
const TITLE_HEIGHT: f32 = 64.0;
const NOTICE_HEIGHT: f32 = 100.0;
const SEARCH_HEIGHT: f32 = 56.0;

#[derive(Default)]
struct State {
    /// Whether the example is shown (preview mode only).
    example: bool,
    /// The words of the search; empty means no search.
    query: String,
}

type Shared = Arc<Mutex<State>>;

fn locked(state: &Mutex<State>) -> MutexGuard<'_, State> {
    state.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Ask the window to draw the surface again (a surface is built again at every change; nothing to do outside a window).
fn redraw() {
    if BlincContextState::is_initialized() {
        BlincContextState::get().request_rebuild();
    }
}

/// One screen of the main view: a view of the timeline.
pub struct Main {
    labels: Labels,
    rows: Arc<Vec<Row>>,
    now: i64,
    mode: Mode,
    view: View,
    state: Shared,
}

impl Main {
    /// The view "All" for a server that offers `capabilities`; `example` says whether the example is shown at first (it only
    /// matters in preview mode).
    #[must_use]
    pub fn new(texts: &Mf2Texts, capabilities: &[String], example: bool) -> Self {
        let now = now_minutes();
        Self {
            labels: Labels::read(texts),
            rows: Arc::new(rows(texts, now)),
            now,
            mode: mode(capabilities),
            view: View::All,
            state: Arc::new(Mutex::new(State {
                example,
                query: String::new(),
            })),
        }
    }

    /// Another screen of the same window: the same rows, the same example switch and the same search, another `view`.
    #[must_use]
    pub fn sibling(&self, view: View) -> Self {
        Self {
            labels: self.labels.clone(),
            rows: Arc::clone(&self.rows),
            now: self.now,
            mode: self.mode,
            view,
            state: Arc::clone(&self.state),
        }
    }

    /// Example or real data.
    #[must_use]
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// The view of this screen.
    #[must_use]
    pub fn view(&self) -> View {
        self.view
    }

    /// The title of the screen: the name of the active view.
    #[must_use]
    pub fn title(&self) -> String {
        self.labels.view_name(self.view).to_owned()
    }

    /// The name of any view (for the navigation).
    #[must_use]
    pub fn title_of(&self, view: View) -> &str {
        self.labels.view_name(view)
    }

    /// The marker shown now.
    #[must_use]
    pub fn notice(&self) -> Option<Notice> {
        notice(self.mode, locked(&self.state).example)
    }

    /// The rows shown now.
    #[must_use]
    pub fn items(&self) -> Vec<Row> {
        let state = locked(&self.state);
        visible(
            self.mode,
            state.example,
            &self.rows,
            self.view,
            &state.query,
        )
    }

    /// Show or hide the example (what the button of the marker does).
    pub fn set_example(&self, shown: bool) {
        locked(&self.state).example = shown;
    }

    /// Set the words of the search (what Enter in the search field does).
    pub fn set_query(&self, query: &str) {
        query.clone_into(&mut locked(&self.state).query);
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

    fn search(&self, query: &str) -> impl Element + use<> {
        let field = SearchField::new(&self.labels.search);
        if !query.is_empty() {
            field.set_text_at_end(query);
        }
        let reader = field.clone();
        let state = Arc::clone(&self.state);
        field
            .on_submit(Arc::new(move || {
                locked(&state).query = reader.text();
                redraw();
            }))
            .build()
    }

    /// What stands where the timeline would: the locked view says why, the empty view says what is missing.
    fn empty(&self, query: &str, example: bool, size: (f32, f32)) -> impl Element + use<> {
        let state = Arc::clone(&self.state);
        let nothing_found = self.mode == Mode::Preview && example && !query.trim().is_empty();
        let empty = if self.view.locked() {
            EmptyState::new(&self.labels.why)
                .id("locked")
                .glyph(Glyph::Person)
                .text(&self.labels.who_text)
        } else if nothing_found {
            EmptyState::new(&self.labels.nothing_title)
                .id("nothing")
                .glyph(Glyph::Search)
                .text(&self.labels.nothing_text)
                .action(&self.labels.reset, move || {
                    locked(&state).query.clear();
                    redraw();
                })
        } else {
            EmptyState::new(&self.labels.empty_title)
                .id("empty")
                .glyph(Glyph::History)
                .text(&self.labels.empty_text)
        };
        div().w(size.0).h(size.1).child(empty.build())
    }
}

impl Surface for Main {
    fn build(&self, width: f32, height: f32) -> Box<dyn Element> {
        let (query, example) = {
            let state = locked(&self.state);
            (state.query.clone(), state.example)
        };
        let marker = notice(self.mode, example);
        let items = visible(self.mode, example, &self.rows, self.view, &query);
        let inner = (width - 2.0 * PAD).max(0.0);
        let searchable = !self.view.locked();
        let used = 2.0 * PAD
            + TITLE_HEIGHT
            + marker.map_or(0.0, |_| NOTICE_HEIGHT + GAP)
            + if searchable { SEARCH_HEIGHT + GAP } else { 0.0 };
        let body_size = (inner, (height - used).max(120.0));
        let mut page = div()
            .flex_col()
            .gap(GAP / 4.0)
            .p(PAD / 4.0)
            .w(width)
            .h(height);
        // The title of the screen is the active view, large, like the title of the area in the model.
        page = page.child(
            TextView::new(vec![Block::Heading {
                level: 1,
                text: Inline::plain(self.title()),
            }])
            .build(),
        );
        if let Some(kind) = marker {
            page = page.child(self.marker(kind));
        }
        if searchable {
            page = page.child(self.search(&query));
        }
        Box::new(if items.is_empty() {
            page.child(self.empty(&query, example, body_size))
        } else {
            page.child(agenda(&self.labels, &items, self.now, body_size))
        })
    }

    fn styles(&self) -> String {
        styles()
    }
}
