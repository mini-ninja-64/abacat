use std::{
    mem::MaybeUninit,
    ops::{Deref, Range},
    pin::Pin,
    ptr::null_mut,
    sync::Arc,
};

use abacat_common::{
    error::Spanned,
    types::PossiblyRef,
    ui::{
        builtin::{HIGHLIGHTER, SyntaxHighlightType, basic_theme},
        theme::{Color, Theme},
    },
};
use cxx::{ExternType, UniquePtr};
use cxx_qt::{Constructor, CxxQtType};
use cxx_qt_lib::{QColor, QObjectExt, QPoint, QVariant};

use crate::{
    settings::THEME,
    syntax_highlighter::qobject::{AbacatSyntaxHighlighter, QmlAbacatSyntaxHighlighter},
    util,
};
// unsafe impl ExternType for AbacatSyntaxHighlighterRust {
//     type Id = cxx::type_id!("AbacatSyntaxHighlighter");
//     type Kind = cxx::kind::Trivial;
// }

// unsafe impl ExternType for crate::syntax_highlighter::qobject::AbacatSyntaxHighlighter {
//     type Id = cxx::type_id!("AbacatSyntaxHighlighter");
//     type Kind = cxx::kind::Trivial;
// }

#[cxx_qt::bridge]
pub mod qobject {
    // struct Simples {
    //     x: AbacatSyntaxHighlighter,
    // }
    // unsafe impl ExternType for MyHighlighter {}
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

        include!(<QPoint>);
        type QPoint = cxx_qt_lib::QPoint;

        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;

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

        // include!("helper.h");
        // fn m() -> Simples;
        // unsafe fn m() -> AbacatSyntaxHighlighter;

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
        #[cxx_name = "rehighlight"]
        fn rehighlight(self: Pin<&mut AbacatSyntaxHighlighter>);

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

        #[qobject]
        #[qml_element]
        #[qproperty(QVariant, error_range, READ=get_error_range, WRITE=set_error_range)]
        #[qproperty(bool, render_as_error, READ=get_render_as_error, WRITE=set_render_as_error)]
        #[qproperty(*mut QQuickTextDocument, input_document, READ, WRITE=set_document)]
        type QmlAbacatSyntaxHighlighter = super::QmlAbacatSyntaxHighlighterRust;

        fn set_document(
            self: Pin<&mut QmlAbacatSyntaxHighlighter>,
            document: *mut QQuickTextDocument,
        );

        fn set_error_range(self: Pin<&mut QmlAbacatSyntaxHighlighter>, range: &QVariant);
        fn get_error_range(self: &QmlAbacatSyntaxHighlighter) -> QVariant;

        fn set_render_as_error(self: Pin<&mut QmlAbacatSyntaxHighlighter>, render_as_error: bool);
        fn get_render_as_error(self: &QmlAbacatSyntaxHighlighter) -> bool;
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

#[derive(Debug)]
pub struct AbacatSyntaxHighlighterRust {
    // TODO: THEME
    // error_range: Option<Range<usize>>,
    // render_as_error: bool,
}

impl Default for AbacatSyntaxHighlighterRust {
    fn default() -> Self {
        Self {
            // error_range: None,
            // render_as_error: false,
        }
    }
}

pub struct QmlAbacatSyntaxHighlighterRust {
    highlighter: UniquePtr<AbacatSyntaxHighlighter>,
    input_document: *mut qobject::QQuickTextDocument,
    error_range: Option<Range<usize>>,
    render_as_error: bool,
}

impl Default for QmlAbacatSyntaxHighlighterRust {
    fn default() -> Self {
        Self {
            highlighter: UniquePtr::null(),
            input_document: Default::default(),
            error_range: None,
            render_as_error: false,
        }
    }
}

macro_rules! set_color_for_range {
    ($self: ident, $range: ident, $color_source: expr) => {{
        let qcolor = util::to_qcolor($color_source);
        $self
            .as_mut()
            .set_foreground($range.start as i32, $range.len() as i32, &qcolor);
    }};
}

impl qobject::AbacatSyntaxHighlighter {
    fn highlight_block(mut self: Pin<&mut Self>, text: &qobject::QString) {
        // let string = text.to_string();
        // let theme = THEME.read().unwrap();

        // // Note: Must set plain text on everything, otherwise underlines are rendered as
        // //       the fallback text colour
        // let string_range = 0..string.len() as i32;
        // if self.render_as_error {
        //     set_color_for_range!(self, string_range, &theme.design.error_text);
        // } else {
        //     set_color_for_range!(self, string_range, &theme.design.plain_text);
        //     for (_, syntax_type, range) in HIGHLIGHTER.highlights_iter(string.as_str()) {
        //         match syntax_type {
        //             SyntaxHighlightType::Boolean => {
        //                 set_color_for_range!(self, range, &theme.design.boolean);
        //             }
        //             SyntaxHighlightType::Identifier => {
        //                 set_color_for_range!(self, range, &theme.design.identifier);
        //             }
        //             SyntaxHighlightType::Base2 => {
        //                 set_color_for_range!(self, range, &theme.design.base2);
        //             }
        //             SyntaxHighlightType::Base8 => {
        //                 set_color_for_range!(self, range, &theme.design.base8);
        //             }
        //             SyntaxHighlightType::Base10 => {
        //                 set_color_for_range!(self, range, &theme.design.base10);
        //             }
        //             SyntaxHighlightType::Base10Decimal => {
        //                 set_color_for_range!(self, range, &theme.design.base10_decimal);
        //             }
        //             SyntaxHighlightType::Base16 => {
        //                 set_color_for_range!(self, range, &theme.design.base16);
        //             }
        //             _ => {}
        //         }
        //     }
        // }

        // if let Some(range) = &self.error_range {
        //     let color = util::to_qcolor(&theme.design.error_underline);
        //     let start = range.start as i32;
        //     let count = range.len() as i32;
        //     self.as_mut().set_underline(
        //         start,
        //         count,
        //         &color,
        //         ffi::QTextCharFormatUnderlineStyle::SingleUnderline,
        //     );
        // }
    }
}

impl qobject::QmlAbacatSyntaxHighlighter {
    fn set_document(mut self: Pin<&mut Self>, document: *mut qobject::QQuickTextDocument) {
        self.as_mut().rust_mut().input_document = document;

        let text_document = unsafe {
            let input = Pin::new_unchecked(&mut *document);
            input.text_document()
        };

        let x = qobject::AbacatSyntaxHighlighter::new(());

        // let mut highlighter = unsafe { ffi::make_abacat_syntax_highlighter(text_document) };
        // if let Some(mut highlighter) = highlighter.as_mut() {
        //     highlighter.as_mut().rust_mut().render_as_error = self.render_as_error;
        //     highlighter.as_mut().rust_mut().error_range = self.error_range.clone();
        //     highlighter.rehighlight();
        // }
        // self.as_mut().rust_mut().highlighter = highlighter;
    }

    fn set_error_range(mut self: Pin<&mut Self>, range: &qobject::QVariant) {
        let range = range.value::<QPoint>().map(|range| {
            let start = range.x() as usize;
            let end = range.y() as usize;
            start..end
        });
        self.as_mut().rust_mut().error_range = range.clone();
        if let Some(mut highlighter) = self.as_mut().rust_mut().highlighter.as_mut() {
            // highlighter.as_mut().rust_mut().error_range = range;
            highlighter.rehighlight();
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
        if let Some(mut highlighter) = self.as_mut().rust_mut().highlighter.as_mut() {
            // highlighter.as_mut().rust_mut().render_as_error = render_as_error;
            highlighter.rehighlight();
        }
    }

    fn get_render_as_error(self: &Self) -> bool {
        self.render_as_error
    }
}

impl cxx_qt::Constructor<(*mut qobject::QTextDocument,)> for qobject::AbacatSyntaxHighlighter {
    type NewArguments = ();

    type BaseArguments = (*mut qobject::QTextDocument,);

    type InitializeArguments = ();

    fn route_arguments(
        args: (*mut qobject::QTextDocument,),
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
