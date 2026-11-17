//! The window: name, the real icon (embedded PNG, own design), the content.
//!
//! Pattern: Adapter: our blocks in, a screen of the fs-components shell out; no widget is painted by hand.

use std::sync::Arc;

use fs_components::blinc::layout::div::div;
use fs_components::blinc::layout::image::image;
use fs_components_app::{AppError, AppWidget, Element, Menu, MenuEntry, Screen, Status, Surface};
use fs_components_mf2::Mf2Texts;
use fs_components_widgets::{Block, Inline, TextView, styles};
use fs_components_words::{Args, Words};

use crate::content::{coming, start};
use crate::link::Link;
use crate::main_view::Main;
use crate::preview::Mode;

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
    fn build(&self, _width: f32, _height: f32) -> Box<dyn Element> {
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
        let goal = TextView::new(vec![Block::Paragraph(Inline::plain(self.tagline.as_str()))]).build();
        let body = TextView::new(self.body.clone()).build();
        let page = div().flex_col().gap(12.0).p(24.0);
        Box::new(if self.header {
            page.child(header).child(goal).child(body)
        } else {
            page.child(body)
        })
    }

    fn styles(&self) -> String {
        styles()
    }
}

/// The window for `link`, with the menus whose entries are all locked ("coming soon").
///
/// # Errors
/// The shell could not be made (a name that is empty, for example).
pub fn app(texts: &Mf2Texts, link: &Link) -> Result<AppWidget, AppError> {
    app_with(texts, link, true)
}

/// As [`app`], with the example of the main view shown (`true`) or hidden (`false`, the empty state).
///
/// # Errors
/// The shell could not be made (a name that is empty, for example).
pub fn app_with(texts: &Mf2Texts, link: &Link, example: bool) -> Result<AppWidget, AppError> {
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
    let widget = AppWidget::new(
        title,
        vec![
            Screen::new("main", say("screen-main"), Arc::new(main)),
            Screen::new("home", say("screen-start"), page(true, start(texts, link))),
            Screen::new("coming", say("screen-coming"), page(false, coming(texts))),
        ],
    )?;
    let locked = |id: &str| {
        MenuEntry::new(id, format!("{} ({soon})", say(&format!("menu-{id}")))).disabled()
    };
    // "By me" and "By others" say why they cannot work yet (the source does not deliver who did it): never silently greyed out.
    let why = say("who-missing");
    let without_who =
        |id: &str| MenuEntry::new(id, format!("{} ({why})", say(&format!("menu-{id}")))).disabled();
    widget.set_menus(
        vec![
            Menu::new(
                say("menu-program"),
                vec![locked("new"), locked("accounts"), locked("search")],
            ),
            Menu::new(
                say("menu-filter"),
                vec![locked("all"), without_who("mine"), without_who("others")],
            ),
        ],
        |_| {},
    )?;
    widget.set_status(Some(Status::Info(say("status-ready"))));
    Ok(widget)
}
