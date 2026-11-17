//! The timeline: the rows grouped by day, newest day first and the newest row first inside a day, as a scrolling list in
//! the look of Nextcloud Activity: a large heading per day ("Today", "Yesterday", then the date), a row with the symbol of the
//! event on the left, the sentence with the object in bold and the time on the right.
//!
//! Pattern: Adapter (rows in, plain blocks of the shell out). The agenda of the `calendar-view` can only list oldest first
//! and has no symbols, so this list is made here (gap in DESIGN).

use std::sync::{Arc, Mutex};

use fs_components::blinc::layout::div::{Div, div};
use fs_components::blinc::layout::widgets::scroll::{Scroll, ScrollPhysics};
use fs_components_widgets::{
    Block, CalendarNames, Day, Inline, Mark, MarkKind, TextView, Tone, icon,
};

use crate::labels::Labels;
use crate::preview::{DAY, Row};

/// Width of the column of the symbol and of the column of the time, in pixels.
const ICON_COLUMN: f32 = 32.0;
const TIME_COLUMN: f32 = 110.0;
/// Height of a row (room to breathe, in pixels).
const ROW_HEIGHT: f32 = 44.0;

/// The days of `rows` (as day numbers since 1970-01-01 UTC), the newest day first and inside a day the newest row first (a
/// timeline reads from now backwards).
#[must_use]
pub fn newest_first(rows: &[Row]) -> Vec<(i64, Vec<Row>)> {
    let mut sorted = rows.to_vec();
    sorted.sort_by_key(|r| std::cmp::Reverse(r.at));
    let mut days: Vec<(i64, Vec<Row>)> = Vec::new();
    for row in sorted {
        let day = row.at.div_euclid(DAY);
        match days.last_mut() {
            Some((d, list)) if *d == day => list.push(row),
            _ => days.push((day, vec![row])),
        }
    }
    days
}

/// The heading of a day: the word for today, the word for yesterday, else the date.
#[must_use]
pub fn day_title(names: &CalendarNames, today: i64, day: i64, words: (&str, &str)) -> String {
    match today - day {
        0 => words.0.to_owned(),
        1 => words.1.to_owned(),
        _ => names.date_text(Day::from_number(day)),
    }
}

/// The sentence of a row with its object name in bold.
fn sentence(row: &Row) -> Inline {
    let marks = row
        .text
        .find(&row.name)
        .map(|byte| {
            let start = row.text[..byte].chars().count();
            Mark::new(MarkKind::Strong, start, start + row.name.chars().count())
        })
        .into_iter()
        .collect();
    Inline::new(row.text.clone(), marks)
}

fn line(row: &Row, text_width: f32) -> Div {
    div()
        .flex_row()
        .items_center()
        .gap(3.0)
        .h(ROW_HEIGHT)
        .child(
            div()
                .w(ICON_COLUMN)
                .flex_shrink_0()
                .child(icon(row.glyph, 20.0, Tone::Muted)),
        )
        .child(
            div()
                .w(text_width)
                .child(TextView::new(vec![Block::Paragraph(sentence(row))]).build()),
        )
        .child(
            div().w(TIME_COLUMN).flex_shrink_0().child(
                TextView::new(vec![Block::Paragraph(Inline::plain(row.age.clone()))]).build(),
            ),
        )
}

/// The timeline for `rows` as of `now` (minutes since 1970-01-01 UTC), filling `width` x `height` and scrolling.
#[must_use]
pub fn agenda(labels: &Labels, rows: &[Row], now: i64, (width, height): (f32, f32)) -> Div {
    let today = now.div_euclid(DAY);
    let text_width = (width - ICON_COLUMN - TIME_COLUMN - 64.0).max(120.0);
    let mut column = div().flex_col().w((width - 12.0).max(0.0)).p(2.0);
    for (day, list) in newest_first(rows) {
        let title = day_title(
            &labels.names,
            today,
            day,
            (&labels.today, &labels.yesterday),
        );
        column = column.child(
            TextView::new(vec![Block::Heading {
                level: 2,
                text: Inline::plain(title),
            }])
            .build(),
        );
        for row in &list {
            column = column.child(line(row, text_width));
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
    use super::{day_title, newest_first, sentence};
    use crate::preview::{DAY, rows};
    use crate::words::words;
    use fs_components_widgets::{CalendarNames, MarkKind};
    use fs_components_words::Locale;

    const NOW: i64 = 29_000_000 + 600;

    #[test]
    fn the_newest_day_and_the_newest_row_stand_on_top() {
        let list = rows(&words(&Locale::english()), NOW);
        let days = newest_first(&list);
        assert_eq!(days.len(), 3);
        assert!(
            days.windows(2).all(|w| w[0].0 > w[1].0),
            "days: newest first"
        );
        for (_, rows) in &days {
            assert!(
                rows.windows(2).all(|w| w[0].at >= w[1].at),
                "rows inside a day: newest first"
            );
        }
        assert_eq!(days[0].1[0].age, "11 min ago");
        assert_eq!(days.iter().map(|(_, r)| r.len()).sum::<usize>(), 12);
    }

    fn names() -> CalendarNames {
        CalendarNames::new(
            std::array::from_fn(|i| format!("d{i}")),
            std::array::from_fn(|i| format!("m{i}")),
            "all day",
            "empty",
        )
    }

    #[test]
    fn each_day_has_a_large_heading_today_yesterday_then_the_date() {
        let today = NOW.div_euclid(DAY);
        let words = ("Today", "Yesterday");
        assert_eq!(day_title(&names(), today, today, words), "Today");
        assert_eq!(day_title(&names(), today, today - 1, words), "Yesterday");
        let older = day_title(&names(), today, today - 3, words);
        assert!(older.contains("2026") || older.contains("20"), "{older}");
        assert_ne!(older, "Today");
    }

    #[test]
    fn the_object_name_is_bold_inside_the_sentence_and_nothing_else() {
        let list = rows(&words(&Locale::english()), NOW);
        for row in &list {
            let inline = sentence(row);
            assert_eq!(inline.marks.len(), 1);
            let m = &inline.marks[0];
            assert_eq!(m.kind, MarkKind::Strong);
            let bold: String = inline
                .text
                .chars()
                .skip(m.start)
                .take(m.end - m.start)
                .collect();
            assert_eq!(bold, row.name);
        }
    }
}
