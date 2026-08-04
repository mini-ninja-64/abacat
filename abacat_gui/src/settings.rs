use std::sync::{LazyLock, RwLock};

use abacat_common::ui::{builtin::basic_theme, theme::Theme};
use cxx_qt_lib::{QColor, QString};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {}
    unsafe extern "RustQt" {
        #[qobject]
        type DesignQt = super::DesignQtRust;

        #[qobject]
        type ThemeQt = super::ThemeQtRust;

        #[qobject]
        type Settings = super::SettingsRust;
    }
}

#[derive(Default, Debug)]
pub struct DesignQtRust {
    plain_text: QColor,
    background: QColor,
    answer_opacity: QColor,
    identifier: QColor,
    boolean: QColor,
    base16: QColor,
    base2: QColor,
    base8: QColor,
    base10: QColor,
    base10_decimal: QColor,
}

#[derive(Default, Debug)]
pub struct ThemeQtRust {
    name: QString,
    author: QString,
    description: QString,
    design: *mut qobject::DesignQt,
}

#[derive(Default, Debug)]
pub struct SettingsRust {
    theme: *mut qobject::ThemeQt,
    text_size: usize,
    font: String,
}

pub static THEME: LazyLock<RwLock<Theme>> = LazyLock::new(|| RwLock::new(basic_theme()));
