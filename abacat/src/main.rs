mod cxxqt_object;
mod settings;
mod syntax_highlighter;
use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QUrl};

// TODO: BUGS
// - empty lines are thinner than lines with content by like 1 or 2 pixels :sob:
// - Linux freezes when going from 10000 lines of error to 10000 lines having a value, but updating a value seems ok?

fn main() {
    let mut app = QGuiApplication::new();
    let mut engine = QQmlApplicationEngine::new();

    engine
        .as_mut()
        .expect("ENGINE COULD NOT BE LOADED")
        .load(&QUrl::from("qrc:/main.qml"));

    app.as_mut()
        .expect("APP COULD NOT BE OBTAINED MUTABLY")
        .exec();
}
