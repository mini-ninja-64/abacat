use std::fmt::Display;

use ariadne::{Color, Label, Report, ReportKind};
use chumsky::error::Rich;

pub fn generate_reports<'a, I: 'a>(
    source_name: &str,
    errors: impl ExactSizeIterator<Item = &'a Rich<'a, I>> + DoubleEndedIterator,
) -> Vec<ariadne::Report<'a, (&str, std::ops::Range<usize>)>>
where
    I: Display,
{
    return errors
        .into_iter()
        .map(|e| {
            let mut range = e.span().into_range();
            // TODO: FIGURE OUT WHY THIS HAPPENS INSTEAD OF HACKY WORKAROUND
            //       might be chumksy bug, might be my parser bug
            if range.start > range.end {
                range = range.start..range.start;
            }
            Report::build(ReportKind::Error, (source_name, range.clone()))
                .with_message(e)
                .with_label(
                    Label::new((source_name, range))
                        .with_message(e.reason())
                        .with_color(Color::Red),
                )
                .with_labels(e.contexts().map(|(pat, span)| {
                    Label::new((source_name, span.into_range()))
                        .with_message(format!("while parsing {}", pat))
                }))
                .finish()
        })
        .collect::<Vec<_>>();
}
