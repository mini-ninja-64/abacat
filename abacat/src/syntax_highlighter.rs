use std::pin::Pin;

use abacat_common::ui::{
    builtin::{HIGHLIGHTER, SyntaxHighlightType, basic_theme},
    theme::Theme,
};
use cxx::UniquePtr;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QColor;

use crate::{settings::SETTINGS, syntax_highlighter::ffi::AbacatSyntaxHighlighter};

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;

        include!(< QColor >);
        type QColor = cxx_qt_lib::QColor;

        include!(< QTextCharFormat >);
        type QTextCharFormat;

        include!(< QFont >);
        type QFont = cxx_qt_lib::QFont;
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
        include!("cxx-qt-lib/common.h");

        include!(<QSyntaxHighlighter>);
        #[qobject]
        type QSyntaxHighlighter;

        #[rust_name = "make_abacat_syntax_highlighter"]
        #[namespace = "rust::cxxqtlib1"]
        unsafe fn make_unique(
            text_document: *mut QTextDocument,
        ) -> UniquePtr<AbacatSyntaxHighlighter>;
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[base = QSyntaxHighlighter]
        type AbacatSyntaxHighlighter = super::AbacatSyntaxHighlighterRust;

        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "highlightBlock"]
        fn highlight_block(self: Pin<&mut AbacatSyntaxHighlighter>, text: &QString);

        #[inherit]
        #[cxx_name = "setFormat"]
        fn set_color(
            self: Pin<&mut AbacatSyntaxHighlighter>,
            start: i32,
            count: i32,
            color: &QColor,
        );

        // #[inherit]
        // #[cxx_name = "setFormat"]
        // fn set_char_format(
        //     self: Pin<&mut AbacatSyntaxHighlighter>,
        //     start: i32,
        //     end: i32,
        //     format: &QTextCharFormat,
        // );

        fn set_document(
            self: Pin<&mut QmlAbacatSyntaxHighlighter>,
            document: *mut QQuickTextDocument,
        );

        #[qobject]
        #[qml_element]
        #[qproperty(*mut QQuickTextDocument, input_document, READ, NOTIFY, WRITE=set_document)]
        // #[qproperty(*mut QQuickTextDocument, output_document, READ, NOTIFY, WRITE=set_document)]
        type QmlAbacatSyntaxHighlighter = super::QmlAbacatSyntaxHighlighterRust;
    }

    impl
        cxx_qt::Constructor<
            (*mut QTextDocument,),
            BaseArguments = (*mut QTextDocument,),
            NewArguments = (),
            InitializeArguments = (),
        > for AbacatSyntaxHighlighter
    {
    }

    impl UniquePtr<QTextDocument> {}
    impl UniquePtr<AbacatSyntaxHighlighter> {}
}

pub struct QmlAbacatSyntaxHighlighterRust {
    highlighter: UniquePtr<AbacatSyntaxHighlighter>,
    input_document: *mut ffi::QQuickTextDocument,
}

impl Default for QmlAbacatSyntaxHighlighterRust {
    fn default() -> Self {
        Self {
            highlighter: UniquePtr::null(),
            input_document: Default::default(),
        }
    }
}

impl ffi::QmlAbacatSyntaxHighlighter {
    fn set_document(mut self: Pin<&mut Self>, document: *mut ffi::QQuickTextDocument) {
        self.as_mut().rust_mut().input_document = document;

        let text_document = unsafe {
            let input = Pin::new_unchecked(&mut *document);
            input.text_document()
        };

        self.as_mut().rust_mut().highlighter =
            unsafe { ffi::make_abacat_syntax_highlighter(text_document) };

        self.as_mut().input_document_changed();
    }
}

macro_rules! set_color_for_range {
    ($self: ident, $range: ident, $color_source: expr) => {{
        let color = $color_source;
        let qcolor = QColor::from_rgb(color.red as i32, color.green as i32, color.blue as i32);
        $self
            .as_mut()
            .set_color($range.start as i32, $range.len() as i32, &qcolor);
    }};
}

impl ffi::AbacatSyntaxHighlighter {
    fn highlight_block(mut self: Pin<&mut AbacatSyntaxHighlighter>, text: &ffi::QString) {
        let string = text.to_string();
        // let y = x.background.blue;
        let settings = SETTINGS.lock().unwrap();

        for (_, syntax_type, range) in HIGHLIGHTER.highlights_iter(string.as_str()) {
            match syntax_type {
                SyntaxHighlightType::Boolean => {
                    set_color_for_range!(self, range, &settings.theme.design.boolean);
                }
                SyntaxHighlightType::Identifier => {
                    set_color_for_range!(self, range, &settings.theme.design.identifier);
                }
                SyntaxHighlightType::Base2 => {
                    set_color_for_range!(self, range, &settings.theme.design.base2);
                }
                SyntaxHighlightType::Base8 => {
                    set_color_for_range!(self, range, &settings.theme.design.base8);
                }
                SyntaxHighlightType::Base10 => {
                    set_color_for_range!(self, range, &settings.theme.design.base10);
                }
                SyntaxHighlightType::Base10Decimal => {
                    set_color_for_range!(self, range, &settings.theme.design.base10_decimal);
                }
                SyntaxHighlightType::Base16 => {
                    set_color_for_range!(self, range, &settings.theme.design.base16);
                }
                SyntaxHighlightType::Other => {
                    set_color_for_range!(self, range, &settings.theme.design.plain_text);
                }
            }
        }
    }
}

// TODO: Make common, instead of duplicating for every line
pub struct AbacatSyntaxHighlighterRust {}

impl<'a> Default for AbacatSyntaxHighlighterRust {
    fn default() -> Self {
        Self {}
    }
}

impl cxx_qt::Constructor<(*mut ffi::QTextDocument,)> for ffi::AbacatSyntaxHighlighter {
    type NewArguments = ();

    type BaseArguments = (*mut ffi::QTextDocument,);

    type InitializeArguments = ();

    fn route_arguments(
        args: (*mut ffi::QTextDocument,),
    ) -> (
        Self::NewArguments,
        Self::BaseArguments,
        Self::InitializeArguments,
    ) {
        ((), args, ())
    }

    fn new(_: Self::NewArguments) -> <Self as cxx_qt::CxxQtType>::Rust {
        AbacatSyntaxHighlighterRust::default()
    }
}
