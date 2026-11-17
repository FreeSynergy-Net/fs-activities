//! What the main view shows for a filter and a mode: the marker, the rows that are visible. Pure functions, so a test can read
//! them.
//!
//! Pattern: Strategy (mode and filter decide the rows); no screen element in here.

use fs_components_widgets::CalendarItem;

use crate::preview::{Mode, Program};

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

#[cfg(test)]
mod tests {
    use super::{Filter, Notice, notice, visible};
    use crate::preview::Mode;

    #[test]
    fn the_marker_exists_exactly_in_preview_mode() {
        assert_eq!(notice(Mode::Preview, true), Some(Notice::Shown));
        assert_eq!(notice(Mode::Preview, false), Some(Notice::Hidden));
        assert_eq!(notice(Mode::Live, true), None);
        assert_eq!(notice(Mode::Live, false), None);
    }

    #[test]
    fn nothing_is_filtered_by_default() {
        assert!(Filter::default().is_empty());
        assert_eq!(visible(Mode::Live, true, &[], &Filter::default()).len(), 0);
    }
}
