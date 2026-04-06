use std::{cell::LazyCell, ops::Range};

use regex::{Captures, Regex};

use crate::ui::theme::{Color, Design, Theme};

pub enum SyntaxHighlightType {
    Boolean,
    Identifier,
    Base2,
    Base8,
    Base10,
    Base10Decimal,
    Base16,
    Other,
}

pub struct Highlighter {
    regex: Regex,
}

pub const HIGHLIGHTER: LazyCell<Highlighter> = LazyCell::new(|| {
    Highlighter { 
        regex: Regex::new(r"(true|false)|([a-zA-Z_][a-zA-Z0-9_]*)|(0x[0-9a-fA-F]+)|(0b[10]+)|(0o[0-7]+)|([0-9]+\.[0-9]+)|([0-9]*)").unwrap()
    }
});

impl Highlighter {
    pub fn highlights_iter<'a: 'b, 'b>(&'a self, haystack: &'b str) -> impl Iterator<Item = (&'b str, SyntaxHighlightType, Range<usize>)> {
        self.regex.captures_iter(haystack).map(|caps| {
        let m = caps.get_match();
        let str = caps.get_match().as_str();
        let highlight = if caps.get(1).is_some() {
            SyntaxHighlightType::Boolean
        } else if caps.get(2).is_some() {
            SyntaxHighlightType::Identifier
        } else if caps.get(3).is_some() {
            SyntaxHighlightType::Base16
        } else if caps.get(4).is_some() {
            SyntaxHighlightType::Base2
        } else if caps.get(5).is_some() {
            SyntaxHighlightType::Base8
        } else if caps.get(6).is_some() {
            SyntaxHighlightType::Base10Decimal
        } else if caps.get(7).is_some() {
            SyntaxHighlightType::Base10
        } else {
            SyntaxHighlightType::Other
        };
        (str, highlight, m.range())
    })
    }
}

pub const DEFAULT_THEME: LazyCell<Theme> = LazyCell::new(|| {
    basic_theme()
});

pub fn basic_theme() -> Theme {
    Theme {
        name: "basic".into(),
        author: "abacat".into(),
        description: "todo".into(),
        design: Design {
            plain_text: Color::new(238, 238, 238),
            background: Color::new(70, 70, 70),
            answer_opacity: 0.5,
            identifier: Color::new(0, 245, 33),
            boolean: Color::new(255, 145, 33),
            base16: Color::new(255, 138, 255),
            base2: Color::new(255, 138, 255),
            base8: Color::new(255, 138, 255),
            base10: Color::new(255, 138, 255),
            base10_decimal: Color::new(255, 138, 255),
        },
    }
}
