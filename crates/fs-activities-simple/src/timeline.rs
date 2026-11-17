//! The timeline: the days of the rows, newest first, as a scrolling list in the look of the `calendar-view` agenda.
//!
//! Pattern: Adapter (rows in, plain blocks of the shell out; the classes are those of the agenda, so the theme styles them).

use std::sync::{Arc, Mutex};

use fs_components::blinc::layout::div::{Div, div};
use fs_components::blinc::layout::prelude::text;
use fs_components::blinc::layout::widgets::scroll::{Scroll, ScrollPhysics};
use fs_components_widgets::{CalendarItem, CalendarNames, Day};

use crate::preview::newest_day;

/// The days of `items`, the newest day first and inside a day the newest row first (a timeline reads from now backwards).
/// The agenda of the `calendar-view` can only list oldest first, so the order is made here (gap in DESIGN).
#[must_use]
pub fn newest_first(items: &[CalendarItem]) -> Vec<(Day, Vec<CalendarItem>)> {
    let mut sorted = items.to_vec();
    sorted.sort_by_key(|a| std::cmp::Reverse(a.start));
    let mut days: Vec<(Day, Vec<CalendarItem>)> = Vec::new();
    for item in sorted {
        match days.last_mut() {
            Some((day, rows)) if *day == item.start.day => rows.push(item),
            _ => days.push((item.start.day, vec![item])),
        }
    }
    days
}

/// The timeline: a heading per day and a line per row, with the classes of the `calendar-view` agenda (same look), newest
/// first, scrolling.
pub fn agenda(names: &CalendarNames, items: &[CalendarItem], (width, height): (f32, f32)) -> Div {
    const LINE: f32 = 30.0;
    const TIME: f32 = 64.0;
    let mut column = div().flex_col().w((width - 12.0).max(0.0)).p(2.0);
    for (day, rows) in newest_first(items) {
        let mut heading = div()
            .class("fs-calendar-agenda-day")
            .flex_row()
            .items_center()
            .h(LINE)
            .child(text(names.date_text(day)).no_wrap().no_cursor());
        if day == newest_day() {
            heading = heading.class("fs-calendar-agenda-today");
        }
        column = column.child(heading);
        for item in rows {
            let mut chip = div()
                .class("fs-calendar-item")
                .class(format!("fs-calendar-item-c{}", item.category % 6))
                .flex_row()
                .items_center()
                .gap(1.0)
                .h(LINE - 4.0)
                .overflow_clip();
            for mark in &item.marks {
                chip = chip.child(
                    div()
                        .class("fs-calendar-mark")
                        .flex_shrink_0()
                        .child(text(mark.clone()).no_wrap().no_cursor()),
                );
            }
            chip = chip.child(text(item.title.clone()).no_wrap().no_cursor());
            column = column.child(
                div()
                    .class("fs-calendar-agenda-row")
                    .flex_row()
                    .items_center()
                    .h(LINE)
                    .child(
                        div()
                            .class("fs-calendar-agenda-time")
                            .w(TIME)
                            .flex_shrink_0()
                            .child(text(item.start.clock()).no_wrap().no_cursor()),
                    )
                    .child(chip),
            );
        }
    }
    let physics = Arc::new(Mutex::new(ScrollPhysics::default()));
    div().w(width).h(height).child(
        Scroll::with_physics(physics)
            .w(width)
            .h(height)
            .content(column),
    )
}

#[cfg(test)]
mod tests {
    use super::newest_first;
    use crate::main_view::Main;
    use crate::words::words;
    use fs_components_words::Locale;

    #[test]
    fn the_newest_day_and_the_newest_row_stand_on_top() {
        let items = Main::new(&words(&Locale::english()), &[], true).items();
        let days = newest_first(&items);
        assert_eq!(days.len(), 3);
        assert_eq!(days[0].0.to_string(), "2026-11-17");
        assert_eq!(days[2].0.to_string(), "2026-11-15");
        assert!(
            days.windows(2).all(|w| w[0].0 > w[1].0),
            "days: newest first"
        );
        for (_, rows) in &days {
            assert!(
                rows.windows(2).all(|w| w[0].start >= w[1].start),
                "rows inside a day: newest first"
            );
        }
        assert_eq!(days[0].1[0].start.clock(), "11:10");
        assert_eq!(days.iter().map(|(_, r)| r.len()).sum::<usize>(), 12);
    }
}
