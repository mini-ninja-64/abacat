use std::{ops::Range, pin::Pin};

use abacat_common::{
    error::Spanned,
    ui::{
        builtin::{HIGHLIGHTER, SyntaxHighlightType, basic_theme},
        theme::{Color, Theme},
    },
};
use cxx::UniquePtr;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QColor;

use crate::{settings::THEME, syntax_highlighter::ffi::AbacatSyntaxHighlighter};

#[cxx_qt::bridge]
pub mod ffi {
    #[repr(i32)]
    #[derive(Debug)]
    pub enum QTextCharFormatUnderlineStyle {
        NoUnderline = 0,
        SingleUnderline = 1,
        DashUnderline = 2,
        DotLine = 3,
        DashDotLine = 4,
        DashDotDotLine = 5,
        WaveUnderline = 6,
        SpellCheckUnderline = 7,
    }

    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;

        include!(< QColor >);
        type QColor = cxx_qt_lib::QColor;

        include!(< QFont >);
        type QFont = cxx_qt_lib::QFont;

        include!("helper.h");
        type QTextCharFormatUnderlineStyle;
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

        include!("helper.h");
        #[qobject]
        type QSyntaxHighlighterUtil;

        #[rust_name = "make_abacat_syntax_highlighter"]
        #[namespace = "rust::cxxqtlib1"]
        unsafe fn make_unique(
            text_document: *mut QTextDocument,
        ) -> UniquePtr<AbacatSyntaxHighlighter>;
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[base = QSyntaxHighlighterUtil]
        type AbacatSyntaxHighlighter = super::AbacatSyntaxHighlighterRust;

        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "highlightBlock"]
        fn highlight_block(self: Pin<&mut AbacatSyntaxHighlighter>, text: &QString);

        #[inherit]
        #[cxx_name = "setForeground"]
        fn set_foreground(
            self: Pin<&mut AbacatSyntaxHighlighter>,
            start: i32,
            count: i32,
            color: &QColor,
        );

        #[inherit]
        #[cxx_name = "setBackground"]
        fn set_background(
            self: Pin<&mut AbacatSyntaxHighlighter>,
            start: i32,
            count: i32,
            color: &QColor,
        );

        #[inherit]
        #[cxx_name = "setUnderline"]
        fn set_underline(
            self: Pin<&mut AbacatSyntaxHighlighter>,
            start: i32,
            count: i32,
            color: &QColor,
            underline: QTextCharFormatUnderlineStyle,
        );

        fn set_document(
            self: Pin<&mut QmlAbacatSyntaxHighlighter>,
            document: *mut QQuickTextDocument,
        );

        // fn get_error_highlight(self: &QmlAbacatSyntaxHighlighter) -> QVariant;
        // fn set_error_highlight(self: Pin<&mut QmlAbacatSyntaxHighlighter>, highlight: QVariant);

        #[qobject]
        #[qml_element]
        #[qproperty(*mut QQuickTextDocument, input_document, READ, NOTIFY, WRITE=set_document)]
        // #[qproperty(QVariant, error_highlight, READ=get_error_highlight, WRITE=set_error_highlight)]
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
    error_highlight: Option<Range<usize>>,
}

impl Default for QmlAbacatSyntaxHighlighterRust {
    fn default() -> Self {
        Self {
            highlighter: UniquePtr::null(),
            input_document: Default::default(),
            error_highlight: None,
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
            .set_foreground($range.start as i32, $range.len() as i32, &qcolor);
    }};
}

impl ffi::AbacatSyntaxHighlighter {
    fn highlight_block(mut self: Pin<&mut AbacatSyntaxHighlighter>, text: &ffi::QString) {
        // let mut char_format = ffi::make_text_char_format();
        let string = text.to_string();
        // let y = x.background.blue;
        let theme = THEME.read().unwrap();

        // Note: Must set plain text on everything, otherwise underlines are rendered as
        //       the fallback text colour
        let string_range = 0..string.len() as i32;
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

        // let red = QColor::from_rgb(255, 0, 0);
        // self.as_mut().set_foreground(
        //     0,
        //     text.len() as i32,
        //     &red,
        //     // ffi::QTextCharFormatUnderlineStyle::WaveUnderline,
        // );
        let red = QColor::from_rgb(255, 0, 0);
        self.set_underline(
            0,
            10,
            &red,
            ffi::QTextCharFormatUnderlineStyle::WaveUnderline,
        );
        // format(int) -> QTextCharFormat
        // Need to calculate overlaps
        // textCharFormat.merge(textCharformat)

        // for (style, span) in self.styles {
        //     let format = ffi::QTextCharFormat.new();
        //     self.set_char_format(span.start as i32, span.len() as i32, format);
        // }
    }
}

pub struct OverlayStyle {
    highlight: Color,
}

pub struct AbacatSyntaxHighlighterRust {
    styles: Vec<Spanned<OverlayStyle>>,
}

impl<'a> Default for AbacatSyntaxHighlighterRust {
    fn default() -> Self {
        Self { styles: vec![] }
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
