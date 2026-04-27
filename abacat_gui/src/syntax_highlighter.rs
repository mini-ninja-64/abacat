use std::{ops::Range, pin::Pin};

use abacat_common::ui::builtin::{HIGHLIGHTER, SyntaxHighlightType};
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QPoint, QVariant};

use crate::{
    settings::THEME,
    syntax_highlighter::qobject::{QQuickTextDocument, QmlAbacatSyntaxHighlighter2},
    util,
};

use crate::qtextcharformat::{QTextCharFormat, QTextCharFormatUnderlineStyle};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;

        include!(< QColor >);
        type QColor = cxx_qt_lib::QColor;

        include!(< QFont >);
        type QFont = cxx_qt_lib::QFont;

        include!(<QPoint>);
        type QPoint = cxx_qt_lib::QPoint;

        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;

        include!("qtextcharformat.hpp");
        type QTextCharFormatUnderlineStyle = super::QTextCharFormatUnderlineStyle;

        #[cxx_name = "QTextCharFormatPatched"]
        type QTextCharFormat = super::QTextCharFormat;

    }

    unsafe extern "C++Qt" {
        include!(< QTextDocument >);
        #[qobject]
        type QTextDocument;

        include!("QtQuick/qquicktextdocument.h");
        #[qobject]
        type QQuickTextDocument;

        #[cxx_name = "textDocument"]
        fn text_document(self: &QQuickTextDocument) -> *mut QTextDocument;
    }

    unsafe extern "C++Qt" {
        include!("syntax_highlighter.hpp");
        #[qobject]
        type QSyntaxHighlighterPatched;
    }

    unsafe extern "RustQt" {
        #[inherit]
        #[cxx_name = "rehighlight"]
        fn rehighlight(self: Pin<&mut QmlAbacatSyntaxHighlighter2>);

        #[inherit]
        #[cxx_name = "setDocument"]
        unsafe fn set_document_internal_cpp(
            self: Pin<&mut QmlAbacatSyntaxHighlighter2>,
            doc: *mut QTextDocument,
        );

        fn set_document(self: Pin<&mut QmlAbacatSyntaxHighlighter2>, doc: *mut QQuickTextDocument);

        #[qobject]
        #[qml_element]
        #[base = QSyntaxHighlighterPatched]
        #[qproperty(QVariant, error_range, READ=get_error_range, WRITE=set_error_range)]
        #[qproperty(bool, render_as_error, READ=get_render_as_error, WRITE=set_render_as_error)]
        #[qproperty(*mut QQuickTextDocument, document, READ, WRITE=set_document)]
        type QmlAbacatSyntaxHighlighter2 = super::QmlAbacatSyntaxHighlighter2Rust;

        fn set_error_range(self: Pin<&mut QmlAbacatSyntaxHighlighter2>, range: &QVariant);
        fn get_error_range(self: &QmlAbacatSyntaxHighlighter2) -> QVariant;

        fn set_render_as_error(self: Pin<&mut QmlAbacatSyntaxHighlighter2>, render_as_error: bool);
        fn get_render_as_error(self: &QmlAbacatSyntaxHighlighter2) -> bool;

        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "highlightBlock"]
        fn highlight_block(self: Pin<&mut QmlAbacatSyntaxHighlighter2>, text: &QString);

        // TODO: Remove and bring setFormat into rust now we have qtextcharformat :3

        #[inherit]
        #[cxx_name = "setFormat"]
        fn set_format(
            self: Pin<&mut QmlAbacatSyntaxHighlighter2>,
            start: i32,
            count: i32,
            format: &QTextCharFormat,
        );

        #[inherit]
        #[cxx_name = "mergeFormat"]
        fn merge_format(
            self: Pin<&mut QmlAbacatSyntaxHighlighter2>,
            start: i32,
            count: i32,
            format: &QTextCharFormat,
        );
    }
}

pub struct QmlAbacatSyntaxHighlighter2Rust {
    document: *mut QQuickTextDocument,
    error_range: Option<Range<usize>>,
    render_as_error: bool,
}

impl Default for QmlAbacatSyntaxHighlighter2Rust {
    fn default() -> Self {
        Self {
            document: Default::default(),
            error_range: None,
            render_as_error: false,
        }
    }
}

macro_rules! set_color_for_range {
    ($self: ident, $range: ident, $color_source: expr) => {{
        let qcolor = util::to_qcolor($color_source);
        let mut format = QTextCharFormat::new();
        format.set_foreground_color(&qcolor);
        $self
            .as_mut()
            .set_format($range.start as i32, $range.len() as i32, &format);
    }};
}

impl qobject::QmlAbacatSyntaxHighlighter2 {
    fn set_error_range(mut self: Pin<&mut Self>, range: &qobject::QVariant) {
        let range = range.value::<QPoint>().map(|range| {
            let start = range.x() as usize;
            let end = range.y() as usize;
            start..end
        });
        self.as_mut().rust_mut().error_range = range;
        if !self.document.is_null() {
            self.rehighlight();
        }
    }

    fn get_error_range(self: &Self) -> QVariant {
        self.error_range
            .as_ref()
            .map(|Range { start, end }| QPoint::new(*start as i32, *end as i32))
            .map(|p| Into::<QVariant>::into(&p))
            .unwrap_or_default()
    }

    fn set_render_as_error(mut self: Pin<&mut Self>, render_as_error: bool) {
        self.as_mut().rust_mut().render_as_error = render_as_error;
        if !self.document.is_null() {
            self.rehighlight();
        }
    }

    fn get_render_as_error(self: &Self) -> bool {
        self.render_as_error
    }

    fn set_document(mut self: Pin<&mut QmlAbacatSyntaxHighlighter2>, doc: *mut QQuickTextDocument) {
        self.as_mut().rust_mut().document = doc;
        unsafe {
            let input = Pin::new_unchecked(&mut *doc);
            let text_document = input.text_document();
            self.as_mut().set_document_internal_cpp(text_document);
        };
        self.rehighlight();
    }

    fn highlight_block(mut self: Pin<&mut Self>, text: &qobject::QString) {
        let string = text.to_string();
        let theme = THEME.read().unwrap();

        // Note: Must set plain text on everything, otherwise underlines are rendered as
        //       the fallback text colour
        let string_range = 0..string.len() as i32;
        if self.render_as_error {
            set_color_for_range!(self, string_range, &theme.design.error_text);
        } else {
            set_color_for_range!(self, string_range, &theme.design.plain_text);
            for (_, syntax_type, range) in HIGHLIGHTER.highlights_iter(string.as_str()) {
                match syntax_type {
                    SyntaxHighlightType::Boolean => {
                        set_color_for_range!(self, range, &theme.design.boolean);
                    }
                    SyntaxHighlightType::Identifier => {
                        set_color_for_range!(self, range, &theme.design.identifier);
                    }
                    SyntaxHighlightType::Base2 => {
                        set_color_for_range!(self, range, &theme.design.base2);
                    }
                    SyntaxHighlightType::Base8 => {
                        set_color_for_range!(self, range, &theme.design.base8);
                    }
                    SyntaxHighlightType::Base10 => {
                        set_color_for_range!(self, range, &theme.design.base10);
                    }
                    SyntaxHighlightType::Base10Decimal => {
                        set_color_for_range!(self, range, &theme.design.base10_decimal);
                    }
                    SyntaxHighlightType::Base16 => {
                        set_color_for_range!(self, range, &theme.design.base16);
                    }
                    _ => {}
                }
            }
        }

        if let Some(range) = &self.error_range {
            let color = util::to_qcolor(&theme.design.error_underline);
            let start = range.start as i32;
            let count = range.len() as i32;
            let mut format = QTextCharFormat::new();
            format.set_underline_color(&color);
            format.set_underline_style(QTextCharFormatUnderlineStyle::SingleUnderline);
            self.as_mut().merge_format(start, count, &format);
        }
    }
}
