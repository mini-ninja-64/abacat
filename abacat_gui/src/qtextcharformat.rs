use std::{ffi::c_void, ptr::null_mut};

use cxx::{ExternType, kind};

pub use qobject::QTextCharFormatUnderlineStyle;

#[cxx::bridge]
pub mod qobject {
    #[repr(i32)]
    #[derive(Debug)]
    pub enum QTextFormatFormatType {
        InvalidFormat = -1,
        BlockFormat = 1,
        CharFormat = 2,
        ListFormat = 3,
        FrameFormat = 5,

        UserFormat = 100,
    }

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
        include!(< QColor >);
        type QColor = cxx_qt_lib::QColor;

        include!("qtextcharformat.hpp");
        type QTextCharFormatUnderlineStyle;
        type QTextFormatFormatType;

        #[cxx_name = "QTextCharFormatPatched"]
        pub type QTextCharFormat = super::QTextCharFormat;

        #[cxx_name = "setFontItalic"]
        pub fn set_font_italic(self: &mut QTextCharFormat, italic: bool);

        #[cxx_name = "fontItalic"]
        pub fn font_italic(self: &QTextCharFormat) -> bool;

        #[cxx_name = "setFontKerning"]
        pub fn set_font_kerning(self: &mut QTextCharFormat, kerning: bool);

        #[cxx_name = "fontKerning"]
        pub fn font_kerning(self: &QTextCharFormat) -> bool;

        #[cxx_name = "setUnderlineStyle"]
        pub fn set_underline_style(
            self: &mut QTextCharFormat,
            underline_style: QTextCharFormatUnderlineStyle,
        );

        #[cxx_name = "underlineStyle"]
        pub fn underline_style(self: &QTextCharFormat) -> QTextCharFormatUnderlineStyle;

        #[cxx_name = "setUnderlineColor"]
        pub fn set_underline_color(self: &mut QTextCharFormat, color: &QColor);

        #[cxx_name = "underlineColor"]
        pub fn underline_color(self: &QTextCharFormat) -> QColor;

        #[cxx_name = "setForegroundColor"]
        pub fn set_foreground_color(self: &mut QTextCharFormat, color: &QColor);

        #[cxx_name = "foregroundColor"]
        pub fn foreground_color(self: &QTextCharFormat) -> QColor;

        #[cxx_name = "setBackgroundColor"]
        pub fn set_background_color(self: &mut QTextCharFormat, color: &QColor);

        #[cxx_name = "backgroundColor"]
        pub fn background_color(self: &QTextCharFormat) -> QColor;

        include!("utils.hpp");
        #[cxx_name = "destructor"]
        fn qtextcharformat_destructor(p: &mut QTextCharFormat);
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct QTextCharFormat {
    // QSharedDataPointer, just need something the right size as wont directly access it
    shared_data_pointer: *mut c_void,
    format_type: qobject::QTextFormatFormatType,
}

impl QTextCharFormat {
    pub fn new() -> QTextCharFormat {
        // Note: No need to cross the FFI boundary and call C++ constructor
        //       as simple type
        QTextCharFormat {
            shared_data_pointer: null_mut(),
            format_type: qobject::QTextFormatFormatType::CharFormat,
        }
    }
}

unsafe impl ExternType for QTextCharFormat {
    type Id = cxx::type_id!("QTextCharFormatPatched");
    type Kind = kind::Trivial;
}

// Handle QSharedDataPointer pointer cleanup by calling destructor
impl Drop for QTextCharFormat {
    fn drop(&mut self) {
        qobject::qtextcharformat_destructor(self);
    }
}
