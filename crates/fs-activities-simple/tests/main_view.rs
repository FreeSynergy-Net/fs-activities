//! The main view and its example data: the example lives only in `preview.rs`, is switched off as soon as the server offers
//! real data, and carries the "Preview" marker while it is shown.
//!
//! Pattern: example-based tests over the public API, plus one grep over the sources of this crate.
//! Tests: preview mode, live mode, marker, example only in `preview.rs`.

use std::path::Path;

use fs_activities_simple::{ENTRIES, Filter, ID_PREFIX, Link, Main, Mode, Notice, Program, words};
use fs_components_words::Locale;

fn main_view(caps: &[&str], example: bool) -> Main {
    let caps: Vec<String> = caps.iter().map(|s| (*s).to_owned()).collect();
    Main::new(&words(&Locale::english()), &caps, example)
}

#[test]
fn without_the_capability_the_example_shows_with_the_marker() {
    let view = main_view(&[], true);
    assert_eq!(view.mode(), Mode::Preview);
    assert_eq!(view.notice(), Some(Notice::Shown));
    assert_eq!(view.items().len(), ENTRIES.len());
    assert!(view.items().iter().all(|i| i.key.starts_with(ID_PREFIX)));
}

#[test]
fn with_the_capability_the_example_is_gone_and_so_is_the_marker() {
    let view = main_view(&["history"], true);
    assert_eq!(view.mode(), Mode::Live);
    assert_eq!(view.notice(), None);
    assert_eq!(view.items().len(), 0);
}

#[test]
fn a_connected_server_with_history_is_live_and_one_without_is_preview() {
    let link = |ids: &[&str]| Link::Connected {
        contract: "0.1".into(),
        ids: ids.iter().map(|s| (*s).to_owned()).collect(),
    };
    let ids = |l: &Link| match l {
        Link::Connected { ids, .. } => ids.clone(),
        _ => vec![],
    };
    assert_eq!(fs_activities_simple::mode(&ids(&link(&[]))), Mode::Preview);
    assert_eq!(
        fs_activities_simple::mode(&ids(&link(&["history"]))),
        Mode::Live
    );
}

#[test]
fn the_example_is_there_without_the_capability_and_gone_with_it() {
    let preview = main_view(&[], true);
    assert_eq!(preview.mode(), Mode::Preview);
    assert_eq!(preview.items().len(), 12);
    assert_eq!(preview.notice(), Some(Notice::Shown));
    let live = main_view(&["history"], true);
    assert_eq!(live.mode(), Mode::Live);
    assert!(
        live.items().is_empty(),
        "no example once the server delivers"
    );
    assert_eq!(live.notice(), None);
}

#[test]
fn hiding_the_example_shows_the_empty_state_and_the_marker_stays() {
    let view = main_view(&[], true);
    view.set_example(false);
    assert_eq!(view.items().len(), 0);
    assert_eq!(view.notice(), Some(Notice::Hidden));
    view.set_example(true);
    assert_eq!(view.items().len(), 12);
}

#[test]
fn the_filter_narrows_by_program_and_by_words() {
    let view = main_view(&[], true);
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
    assert_eq!(view.items().len(), 0);
}

#[test]
fn example_data_stand_only_in_preview_rs() {
    let id = ["preview", ":"].concat();
    let name = ["Beispiel", " A"].concat();
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut checked = 0;
    for entry in std::fs::read_dir(&src).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "rs") || path.ends_with("preview.rs") {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        // An example id is the prefix and a number ("preview::" in a `use` line is not one).
        let has_id = text
            .match_indices(&id)
            .any(|(at, _)| text[at + id.len()..].starts_with(|c: char| c.is_ascii_digit()));
        assert!(!has_id, "{} holds example ids", path.display());
        assert!(
            !text.contains(&name),
            "{} holds example names",
            path.display()
        );
        checked += 1;
    }
    assert!(checked >= 8, "only {checked} sources looked at");
}
