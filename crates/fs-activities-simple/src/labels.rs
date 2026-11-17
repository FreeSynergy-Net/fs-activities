//! The words of the main view, read once from the `MessageFormat` 2 texts.
//!
//! Pattern: Value object (pre-resolved strings), so building the screen asks no text twice.

use std::sync::Arc;

use fs_components_mf2::Mf2Texts;
use fs_components_widgets::CalendarNames;

use crate::filter::View;
use crate::preview::Program;

/// The words of the view, read once.
#[derive(Clone)]
pub struct Labels {
    pub preview_title: String,
    pub preview_text: String,
    pub preview_hidden: String,
    pub hide: String,
    pub show: String,
    pub all: String,
    pub programs: Vec<(Program, String)>,
    pub mine: String,
    pub others: String,
    pub why: String,
    pub search: String,
    pub empty_title: String,
    pub empty_text: String,
    pub nothing_title: String,
    pub nothing_text: String,
    pub reset: String,
    pub today: String,
    pub yesterday: String,
    pub who_text: String,
    pub names: Arc<CalendarNames>,
}

impl Labels {
    /// The words of the view in `texts`.
    #[must_use]
    pub fn read(texts: &Mf2Texts) -> Self {
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
            today: texts.plain("day-today"),
            yesterday: texts.plain("day-yesterday"),
            who_text: texts.plain("who-text"),
            names: Arc::new(names),
        }
    }
}

impl Labels {
    /// The name of a view of the navigation.
    #[must_use]
    pub fn view_name(&self, view: View) -> &str {
        match view {
            View::All => &self.all,
            View::ByMe => &self.mine,
            View::ByOthers => &self.others,
            View::Program(p) => self
                .programs
                .iter()
                .find(|(q, _)| *q == p)
                .map_or("", |(_, name)| name.as_str()),
        }
    }
}
