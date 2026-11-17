//! The window: name, the real icon (embedded PNG, own design), the content.
//!
//! Pattern: Adapter: our blocks in, a screen of the fs-components shell out; no widget is painted by hand.

use std::sync::{Arc, Mutex};

use fs_components::blinc::layout::div::div;
use fs_components::blinc::layout::image::image;
use fs_components::blinc::layout::widgets::scroll::{Scroll, ScrollPhysics};
use fs_components_app::{AppError, AppWidget, Element, Menu, MenuEntry, Screen, Status, Surface};
use fs_components_mf2::Mf2Texts;
use fs_components_widgets::{Block, Inline, TextView, styles};
use fs_components_words::{Args, Words};

use crate::content::{coming, start};
use crate::filter::View;
use crate::link::Link;
use crate::main_view::Main;
use crate::preview::{Mode, Program};

/// The icon of the program, embedded in the binary.
const ICON: &[u8] = include_bytes!("../assets/fs-activities.png");

fn icon_uri() -> String {
    use base64::Engine;
    format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(ICON)
    )
}

/// Width of the box of the title, in pixels (the text view spreads over what it gets; a narrow box keeps the title at the icon).
const TITLE_WIDTH: f32 = 340.0;

/// How far the title box is pulled to the icon (Blinc counts margins in units of 4 px; the text view keeps room on its own left).
const TITLE_PULL: f32 = 13.0;

/// Side of the icon in the window, in pixels.
const ICON_SIZE: f32 = 64.0;

/// The start and the "what comes" screen.
struct Home {
    name: String,
    tagline: String,
    body: Vec<Block>,
    /// The start page carries the header with the icon; the page "what comes" is the list alone.
    header: bool,
}

impl Surface for Home {
    fn build(&self, width: f32, height: f32) -> Box<dyn Element> {
        // The icon of the program (own design, embedded; see packaging/ICON.md), as a data address Blinc loads.
        let icon = image(icon_uri()).size(ICON_SIZE, ICON_SIZE).no_fade();
        let title = div().w(TITLE_WIDTH).ml(-TITLE_PULL).child(
            TextView::new(vec![Block::Heading {
                level: 1,
                text: Inline::plain(self.name.as_str()),
            }])
            .build(),
        );
        let header = div()
            .flex_row()
            .items_center()
            .gap(16.0)
            .child(icon)
            .child(title);
        let goal =
            TextView::new(vec![Block::Paragraph(Inline::plain(self.tagline.as_str()))]).build();
        let body = TextView::new(self.body.clone()).build();
        let page = div()
            .flex_col()
            .gap(4.0)
            .p(24.0)
            .w((width - 40.0).max(200.0));
        let column = if self.header {
            page.child(header).child(goal).child(body)
        } else {
            page.child(body)
        };
        // The page scrolls when it is longer than the window, so nothing is cut off at the bottom.
        let physics = Arc::new(Mutex::new(ScrollPhysics::default()));
        Box::new(
            div().w(width).h(height).child(
                Scroll::with_physics(physics)
                    .w(width)
                    .h(height)
                    .content(column),
            ),
        )
    }

    fn styles(&self) -> String {
        styles()
    }
}

/// The symbol of a view in the navigation (characters every font has).
const fn symbol(view: View) -> &'static str {
    match view {
        View::All => "\u{2261}",
        View::ByMe => "\u{263A}",
        View::ByOthers => "\u{263B}",
        View::Program(Program::Texts) => "\u{270E}",
        View::Program(Program::Files) => "\u{25A4}",
        View::Program(Program::Calendars) => "\u{25A6}",
        View::Program(Program::Tasks) => "\u{2713}",
    }
}

/// The window for `link`, with the menus whose entries are all locked ("coming soon"); the example of the main view is shown
/// (`true`) or hidden (`false`, the empty state).
///
/// # Errors
/// The shell could not be made (a name that is empty, for example).
pub fn app(texts: &Mf2Texts, link: &Link, example: bool) -> Result<AppWidget, AppError> {
    let ids: &[String] = match link {
        Link::Connected { ids, .. } => ids,
        Link::Mismatch(_) | Link::Down(_) => &[],
    };
    let main = Main::new(texts, ids, example);
    let preview = main.mode() == Mode::Preview;
    let say = |id: &str| texts.say(&id.into(), &Args::new()).text().to_owned();
    let soon = say("soon");
    let page = |header: bool, body: Vec<Block>| {
        Arc::new(Home {
            name: say("app-name"),
            tagline: say("app-tagline"),
            body,
            header,
        })
    };
    let title = if preview {
        format!("{} {}", say("app-name"), say("title-preview"))
    } else {
        say("app-name")
    };
    // The navigation on the left: the views of the timeline with a symbol each (the screen list takes text only), then the
    // start page and "what comes".
    let mut screens: Vec<Screen> = View::ALL
        .iter()
        .map(|view| {
            let label = format!("{}  {}", symbol(*view), main.title_of(*view));
            Screen::new(view.id(), label, Arc::new(main.sibling(*view)))
        })
        .collect();
    screens.push(Screen::new(
        "home",
        say("screen-start"),
        page(true, start(texts, link)),
    ));
    screens.push(Screen::new(
        "coming",
        say("screen-coming"),
        page(false, coming(texts)),
    ));
    let widget = AppWidget::new(title, screens)?;
    let locked = |id: &str| {
        MenuEntry::new(id, format!("{} ({soon})", say(&format!("menu-{id}")))).disabled()
    };
    widget.set_menus(
        vec![Menu::new(
            say("menu-program"),
            vec![locked("new"), locked("accounts"), locked("search")],
        )],
        |_| {},
    )?;
    widget.set_status(Some(Status::Info(say("status-ready"))));
    Ok(widget)
}
