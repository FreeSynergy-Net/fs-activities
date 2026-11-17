//! The example data of the main view: typed, never stored, never sent, never in the core, the server or the client.
//!
//! The ONLY place of this program that holds example data (a test greps the other sources). The texts of the examples are
//! `MessageFormat` 2 ids (`example-1` ...), names are "Example A/B/C", never a real person. [`mode`] decides whether the
//! window shows these rows or what the server delivers: as soon as the server offers the capability `history`, the example
//! is gone.
//!
//! Pattern: Value object (the rows) + Strategy switch (`mode`): the one place that tells example from real.

use fs_components_mf2::Mf2Texts;
use fs_components_widgets::{CalendarItem, Day, Moment};

/// Every id of an example row starts with this; a real row never does.
pub const ID_PREFIX: &str = "preview:";

/// The capability whose presence means the server delivers real activities.
pub const CAPABILITY: &str = "history";

/// Days since 1970-01-01 of the oldest example day, 2026-11-15; the examples span three days.
const FIRST_DAY: i64 = 20_772;

/// Whether the main view shows example data or what the server delivers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// The server delivers no activities yet: the example is shown and marked "Preview".
    Preview,
    /// The server delivers activities: no example data, no preview marker.
    Live,
}

/// The program an activity came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Program {
    /// Texts.
    Texts,
    /// Files.
    Files,
    /// Calendars.
    Calendars,
    /// Tasks.
    Tasks,
}

impl Program {
    /// All programs of the example, in the order of the filter.
    pub const ALL: [Program; 4] = [Self::Texts, Self::Files, Self::Calendars, Self::Tasks];

    /// The id of the word that names the program.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Texts => "prog-texts",
            Self::Files => "prog-files",
            Self::Calendars => "prog-calendars",
            Self::Tasks => "prog-tasks",
        }
    }

    /// The colour step of the timeline (the same program, the same colour).
    #[must_use]
    pub const fn category(self) -> u8 {
        match self {
            Self::Texts => 0,
            Self::Files => 1,
            Self::Calendars => 2,
            Self::Tasks => 3,
        }
    }
}

/// One example row: when, from which program, and the id of its words.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    /// The id, with [`ID_PREFIX`].
    pub id: &'static str,
    /// 0 is the oldest example day, 2 the newest.
    pub day: u8,
    /// The time of day.
    pub at: (u16, u16),
    /// The program.
    pub program: Program,
    /// The id of the text.
    pub text: &'static str,
}

/// The twelve example rows over three days; the last one stands for "4 more changes" (a summary).
pub const ENTRIES: [Entry; 12] = [
    row("preview:1", 0, (8, 12), Program::Files, "example-1"),
    row("preview:2", 0, (9, 40), Program::Texts, "example-2"),
    row("preview:3", 0, (10, 5), Program::Tasks, "example-3"),
    row("preview:4", 0, (14, 30), Program::Calendars, "example-4"),
    row("preview:5", 1, (8, 55), Program::Texts, "example-5"),
    row("preview:6", 1, (11, 20), Program::Tasks, "example-6"),
    row("preview:7", 1, (13, 0), Program::Files, "example-7"),
    row("preview:8", 1, (16, 45), Program::Calendars, "example-8"),
    row("preview:9", 2, (7, 50), Program::Tasks, "example-9"),
    row("preview:10", 2, (9, 15), Program::Texts, "example-10"),
    row("preview:11", 2, (10, 30), Program::Files, "example-11"),
    row("preview:12", 2, (11, 10), Program::Texts, "example-12"),
];

const fn row(
    id: &'static str,
    day: u8,
    at: (u16, u16),
    program: Program,
    text: &'static str,
) -> Entry {
    Entry {
        id,
        day,
        at,
        program,
        text,
    }
}

/// Example or real: [`Mode::Live`] exactly when the capability `history` is among `capabilities` (ids as the server reports
/// them, with or without a version after a space).
#[must_use]
pub fn mode(capabilities: &[String]) -> Mode {
    if capabilities
        .iter()
        .any(|id| id.split_whitespace().next() == Some(CAPABILITY))
    {
        Mode::Live
    } else {
        Mode::Preview
    }
}

/// The oldest example day.
#[must_use]
pub fn oldest_day() -> Day {
    Day::from_number(FIRST_DAY)
}

/// The newest example day (the day marked as today in the timeline).
#[must_use]
pub fn newest_day() -> Day {
    Day::from_number(FIRST_DAY + 2)
}

/// The example rows with their words from `texts`: every item carries the word "Example" and the program as marks, so the
/// example is told in words on every row, not by colour alone.
#[must_use]
pub fn rows(texts: &Mf2Texts) -> Vec<(Program, CalendarItem)> {
    let example = texts.plain("example-word");
    ENTRIES
        .iter()
        .map(|e| {
            let at = Moment::at(oldest_day().add_days(i64::from(e.day)), e.at.0, e.at.1);
            let item = CalendarItem::timed(e.id, texts.plain(e.text), at, at)
                .marked(example.clone())
                .marked(texts.plain(e.program.word()))
                .in_category(e.program.category());
            (e.program, item)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{ENTRIES, ID_PREFIX, Mode, Program, mode, newest_day, oldest_day, rows};
    use crate::words::words;
    use fs_components_words::Locale;

    fn ids(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn no_capability_means_the_example_and_history_means_real_data() {
        assert_eq!(mode(&[]), Mode::Preview);
        assert_eq!(mode(&ids(&["files", "calendar"])), Mode::Preview);
        assert_eq!(mode(&ids(&["history"])), Mode::Live);
        assert_eq!(mode(&ids(&["files", "history 0.1"])), Mode::Live);
        // A name that only starts with the word is another capability.
        assert_eq!(mode(&ids(&["historyx"])), Mode::Preview);
    }

    #[test]
    fn the_days_are_what_the_constant_says() {
        assert_eq!(oldest_day().to_string(), "2026-11-15");
        assert_eq!(newest_day().to_string(), "2026-11-17");
    }

    #[test]
    fn twelve_rows_over_three_days_each_with_words_in_both_languages() {
        for locale in [Locale::english(), Locale::parse("de").unwrap()] {
            let texts = words(&locale);
            let list = rows(&texts);
            assert_eq!(list.len(), 12);
            let days: std::collections::BTreeSet<_> =
                list.iter().map(|(_, i)| i.start.day).collect();
            assert_eq!(days.len(), 3);
            for ((_, item), entry) in list.iter().zip(ENTRIES) {
                assert!(item.key.starts_with(ID_PREFIX));
                assert_ne!(item.title, entry.text, "the words are missing");
                assert!(
                    item.title.contains("Example") || item.title.contains("Beispiel"),
                    "{}",
                    item.title
                );
                assert_eq!(item.marks.len(), 2, "the word Example and the program");
            }
        }
    }

    #[test]
    fn every_program_has_rows_and_a_colour_of_its_own() {
        for program in Program::ALL {
            assert!(ENTRIES.iter().any(|e| e.program == program));
        }
        let colours: std::collections::BTreeSet<_> =
            Program::ALL.iter().map(|p| p.category()).collect();
        assert_eq!(colours.len(), Program::ALL.len());
    }
}
