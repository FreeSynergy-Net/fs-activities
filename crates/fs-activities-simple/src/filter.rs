//! What the main view shows for a view (a filter of the navigation), a search and a mode: the marker, the rows that are
//! visible. Pure functions, so a test can read them.
//!
//! Pattern: Strategy (mode, view and search decide the rows); no screen element in here.

use crate::preview::{Mode, Program, Row};

/// A view of the timeline, one entry of the navigation on the left (in the look of Nextcloud Activity).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum View {
    /// All activities.
    All,
    /// Only what I did: locked, the source does not say who did it.
    ByMe,
    /// Only what others did: locked, the source does not say who did it.
    ByOthers,
    /// Only the activities of one program.
    Program(Program),
}

impl View {
    /// All views in the order of the navigation.
    pub const ALL: [View; 7] = [
        Self::All,
        Self::ByMe,
        Self::ByOthers,
        Self::Program(Program::Texts),
        Self::Program(Program::Files),
        Self::Program(Program::Calendars),
        Self::Program(Program::Tasks),
    ];

    /// Whether the view cannot work yet (the source does not deliver who did it).
    #[must_use]
    pub const fn locked(self) -> bool {
        matches!(self, Self::ByMe | Self::ByOthers)
    }

    /// The name of the screen of this view (`--screen`).
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::All => "main",
            Self::ByMe => "mine",
            Self::ByOthers => "others",
            Self::Program(Program::Texts) => "texts",
            Self::Program(Program::Files) => "files",
            Self::Program(Program::Calendars) => "calendars",
            Self::Program(Program::Tasks) => "tasks",
        }
    }

    /// The id of the word that names the view.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::All => "filter-all",
            Self::ByMe => "filter-mine",
            Self::ByOthers => "filter-others",
            Self::Program(p) => p.word(),
        }
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
/// activities yet, so live shows none); none in a locked view; narrowed by the program of the view and by the words of
/// the search (any case).
#[must_use]
pub fn visible(mode: Mode, example: bool, all: &[Row], view: View, query: &str) -> Vec<Row> {
    if mode != Mode::Preview || !example || view.locked() {
        return Vec::new();
    }
    let query = query.trim().to_lowercase();
    all.iter()
        .filter(|row| match view {
            View::Program(p) => row.program == p,
            _ => true,
        })
        .filter(|row| query.is_empty() || row.text.to_lowercase().contains(&query))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{Notice, View, notice, visible};
    use crate::preview::{Mode, Program, rows};
    use crate::words::words;
    use fs_components_words::Locale;

    #[test]
    fn the_marker_exists_exactly_in_preview_mode() {
        assert_eq!(notice(Mode::Preview, true), Some(Notice::Shown));
        assert_eq!(notice(Mode::Preview, false), Some(Notice::Hidden));
        assert_eq!(notice(Mode::Live, true), None);
        assert_eq!(notice(Mode::Live, false), None);
    }

    #[test]
    fn the_views_narrow_the_rows_and_the_locked_ones_show_none() {
        let all = rows(&words(&Locale::english()), 1_000_000);
        assert_eq!(visible(Mode::Preview, true, &all, View::All, "").len(), 12);
        assert_eq!(visible(Mode::Preview, true, &all, View::ByMe, "").len(), 0);
        assert_eq!(
            visible(Mode::Preview, true, &all, View::ByOthers, "").len(),
            0
        );
        let texts = visible(Mode::Preview, true, &all, View::Program(Program::Texts), "");
        assert_eq!(texts.len(), 4);
        assert!(texts.iter().all(|r| r.program == Program::Texts));
        assert_eq!(
            visible(Mode::Preview, true, &all, View::All, "  FOLDER ").len(),
            1
        );
        assert_eq!(
            visible(
                Mode::Preview,
                true,
                &all,
                View::Program(Program::Tasks),
                "folder"
            )
            .len(),
            0
        );
        assert_eq!(visible(Mode::Live, true, &all, View::All, "").len(), 0);
        assert_eq!(visible(Mode::Preview, false, &all, View::All, "").len(), 0);
    }

    #[test]
    fn only_the_two_who_views_are_locked() {
        assert_eq!(View::ALL.iter().filter(|v| v.locked()).count(), 2);
    }
}
