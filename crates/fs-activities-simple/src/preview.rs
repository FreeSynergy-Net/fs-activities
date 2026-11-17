//! The example data of the main view: typed, never stored, never sent, never in the core, the server or the client.
//!
//! The ONLY place of this program that holds example data (a test greps the other sources). The texts of the examples are
//! `MessageFormat` 2 ids (`example-1` ...), the names are "Example A/B/C", never a real person. Times are minutes before
//! "now" in UTC; the rows get their time from the clock they are given, so a test can use a fixed one. [`mode`] decides
//! whether the window shows these rows or what the server delivers: as soon as the server offers the capability `history`,
//! the example is gone.
//!
//! Pattern: Value object (the rows) + Strategy switch (`mode`): the one place that tells example from real.

use fs_components_mf2::Mf2Texts;
use fs_components_widgets::Glyph;
use fs_components_words::{Args, Words};

/// Every id of an example row starts with this; a real row never does.
pub const ID_PREFIX: &str = "preview:";

/// The capability whose presence means the server delivers real activities.
pub const CAPABILITY: &str = "history";

/// Minutes in a day.
pub const DAY: i64 = 1440;

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
    /// All programs of the example, in the order of the navigation.
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
}

/// One example row: how long ago, from which program, with which symbol, and the ids of its words.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    /// The id, with [`ID_PREFIX`].
    pub id: &'static str,
    /// Minutes before now.
    pub ago: i64,
    /// The program.
    pub program: Program,
    /// The symbol of the kind of event (done, created, changed ...).
    pub glyph: Glyph,
    /// The id of the sentence.
    pub text: &'static str,
    /// The id of the name of the object ("Example A").
    pub name: &'static str,
}

const fn row(
    id: &'static str,
    ago: i64,
    program: Program,
    glyph: Glyph,
    (text, name): (&'static str, &'static str),
) -> Entry {
    Entry {
        id,
        ago,
        program,
        glyph,
        text,
        name,
    }
}

/// The twelve example rows: four a few minutes ago, four a day ago, four three days ago; the last one stands for "4 more
/// changes" (a summary).
pub const ENTRIES: [Entry; 12] = [
    row(
        "preview:1",
        11,
        Program::Tasks,
        Glyph::Check,
        ("example-1", "example-name-c"),
    ),
    row(
        "preview:2",
        12,
        Program::Tasks,
        Glyph::Plus,
        ("example-2", "example-name-b"),
    ),
    row(
        "preview:3",
        13,
        Program::Calendars,
        Glyph::Calendar,
        ("example-3", "example-name-a"),
    ),
    row(
        "preview:4",
        14,
        Program::Files,
        Glyph::Upload,
        ("example-4", "example-name-a"),
    ),
    row(
        "preview:5",
        1500,
        Program::Texts,
        Glyph::Edit,
        ("example-5", "example-name-a"),
    ),
    row(
        "preview:6",
        1560,
        Program::Calendars,
        Glyph::Calendar,
        ("example-6", "example-name-b"),
    ),
    row(
        "preview:7",
        1600,
        Program::Files,
        Glyph::Share,
        ("example-7", "example-name-c"),
    ),
    row(
        "preview:8",
        1700,
        Program::Tasks,
        Glyph::Plus,
        ("example-8", "example-name-a"),
    ),
    row(
        "preview:9",
        4400,
        Program::Texts,
        Glyph::Plus,
        ("example-9", "example-name-c"),
    ),
    row(
        "preview:10",
        4420,
        Program::Files,
        Glyph::Rename,
        ("example-10", "example-name-b"),
    ),
    row(
        "preview:11",
        4440,
        Program::Texts,
        Glyph::Edit,
        ("example-11", "example-name-b"),
    ),
    row(
        "preview:12",
        4460,
        Program::Texts,
        Glyph::Edit,
        ("example-12", "example-name-a"),
    ),
];

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

/// Now, in minutes since 1970-01-01 UTC (0 if the clock is before that).
#[must_use]
pub fn now_minutes() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .and_then(|d| i64::try_from(d.as_secs() / 60).ok())
        .unwrap_or(0)
}

/// A row of the timeline, ready to draw: the sentence, the object name inside it (drawn bold), the time.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    /// The id.
    pub id: String,
    /// When, in minutes since 1970-01-01 UTC.
    pub at: i64,
    /// The program.
    pub program: Program,
    /// The symbol of the kind of event.
    pub glyph: Glyph,
    /// The whole sentence.
    pub text: String,
    /// The name of the object; it stands inside `text`.
    pub name: String,
    /// How long ago, relative to the clock the row was made with ("11 min ago").
    pub age: String,
}

/// How long ago `minutes` were, in words ("just now", "11 min ago", "25 h ago", "3 d ago").
#[must_use]
pub fn age(texts: &Mf2Texts, minutes: i64) -> String {
    let say = |id: &str, n: i64| {
        texts
            .say(&id.into(), &Args::new().text("n", n.to_string()))
            .text()
            .to_owned()
    };
    match minutes {
        ..1 => say("ago-now", 0),
        1..60 => say("ago-minutes", minutes),
        60..2880 => say("ago-hours", minutes / 60),
        _ => say("ago-days", minutes / DAY),
    }
}

/// The example rows with their words from `texts`, made at `now` (minutes since 1970-01-01 UTC).
#[must_use]
pub fn rows(texts: &Mf2Texts, now: i64) -> Vec<Row> {
    ENTRIES
        .iter()
        .map(|e| {
            let name = texts.plain(e.name);
            let text = texts
                .say(&e.text.into(), &Args::new().text("name", name.as_str()))
                .text()
                .to_owned();
            Row {
                id: e.id.to_owned(),
                at: now - e.ago,
                program: e.program,
                glyph: e.glyph,
                text,
                name,
                age: age(texts, e.ago),
            }
        })
        .collect()
}
