use std::sync::{LazyLock, Mutex};

use abacat_common::ui::{builtin::basic_theme, theme::Theme};

pub struct Settings {
    pub theme: Theme,
    pub text_size: usize,
    pub font: String,
}

pub static SETTINGS: LazyLock<Mutex<Settings>> = LazyLock::new(|| {
    Mutex::new(Settings {
        theme: basic_theme(),
        text_size: 10,
        font: "Monaco".into(),
    })
});
