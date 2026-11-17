//! The main view and its example data: the example lives only in `preview.rs`, is switched off as soon as the server offers
//! real data, carries the "Preview" marker while it is shown, and the views of the navigation narrow it.
//!
//! Pattern: example-based tests over the public API, plus one grep over the sources of this crate.
//! Tests: preview mode, live mode, marker, views (locked ones), search, example only in `preview.rs`.

use std::path::Path;

use fs_activities_simple::{ENTRIES, ID_PREFIX, Link, Main, Mode, Notice, Program, View, words};
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
    assert!(view.items().iter().all(|i| i.id.starts_with(ID_PREFIX)));
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
fn hiding_the_example_shows_the_empty_state_and_the_marker_stays() {
    let view = main_view(&[], true);
    view.set_example(false);
    assert_eq!(view.items().len(), 0);
    assert_eq!(view.notice(), Some(Notice::Hidden));
    view.set_example(true);
    assert_eq!(view.items().len(), 12);
}

#[test]
fn the_views_narrow_by_program_and_the_who_views_are_locked_with_no_rows() {
    let all = main_view(&[], true);
    let texts = all.sibling(View::Program(Program::Texts));
    assert_eq!(texts.items().len(), 4);
    assert!(texts.items().iter().all(|r| r.program == Program::Texts));
    for locked in [View::ByMe, View::ByOthers] {
        let view = all.sibling(locked);
        assert!(view.view().locked());
        assert_eq!(view.items().len(), 0, "{locked:?} has no rows");
    }
    assert_eq!(View::ALL.len(), 7, "all, by me, by others, four programs");
}

#[test]
fn the_search_is_shared_by_the_screens_and_narrows_by_words() {
    let view = main_view(&[], true);
    view.set_query("  FOLDER ");
    assert_eq!(view.items().len(), 1);
    assert_eq!(view.sibling(View::Program(Program::Files)).items().len(), 1);
    assert_eq!(view.sibling(View::Program(Program::Tasks)).items().len(), 0);
    view.set_query("");
    assert_eq!(view.items().len(), 12);
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
    assert!(checked >= 9, "only {checked} sources looked at");
}
