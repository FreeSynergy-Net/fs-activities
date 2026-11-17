//! The example data of the main view: the mode switch, the relative time with a fixed clock, the rows in both languages.
//!
//! Pattern: example-based tests over the public API.
//! Tests: mode, age, rows.

use fs_activities_simple::{ENTRIES, ID_PREFIX, Mode, Program, age, mode, rows, words};
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
fn the_time_is_relative_to_the_clock_it_is_given() {
    let en = words(&Locale::english());
    assert_eq!(age(&en, 0), "just now");
    assert_eq!(age(&en, 11), "11 min ago");
    assert_eq!(age(&en, 59), "59 min ago");
    assert_eq!(age(&en, 60), "1 h ago");
    assert_eq!(age(&en, 1500), "25 h ago");
    assert_eq!(age(&en, 4400), "3 d ago");
    let de = words(&Locale::parse("de").unwrap());
    assert_eq!(age(&de, 11), "vor 11 Min.");
    let now = 30_000_000;
    let list = rows(&en, now);
    assert_eq!(list[0].at, now - 11);
    assert_eq!(list[0].age, "11 min ago");
}

#[test]
fn twelve_rows_each_with_words_in_both_languages_and_the_name_inside_the_sentence() {
    for locale in [Locale::english(), Locale::parse("de").unwrap()] {
        let texts = words(&locale);
        let list = rows(&texts, 1_000_000);
        assert_eq!(list.len(), 12);
        for (row, entry) in list.iter().zip(ENTRIES) {
            assert!(row.id.starts_with(ID_PREFIX));
            assert_ne!(row.text, entry.text, "the words are missing");
            assert!(
                row.name.contains("Example") || row.name.contains("Beispiel"),
                "{}",
                row.name
            );
            assert!(row.text.contains(&row.name), "{}", row.text);
        }
    }
}

#[test]
fn every_program_has_rows() {
    for program in Program::ALL {
        assert!(ENTRIES.iter().any(|e| e.program == program));
    }
}
