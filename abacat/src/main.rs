mod cxxqt_object;
mod settings;
mod syntax_highlighter;
use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QUrl};

// TODO: BUGS
//   - Syntax parsing failures does not invalidate old eval

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
