use abacat_common::ui::theme::Color;
use cxx_qt_lib::QColor;

pub fn to_qcolor(color: &Color) -> QColor {
    QColor::from_rgb(color.red as i32, color.green as i32, color.blue as i32)
}
