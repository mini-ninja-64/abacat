mod cxxqt_object;
mod experiment;
mod settings;
mod syntax_highlighter;
mod util;
use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QUrl};

// use crate::syntax_highlighter::qobject::test_func;
//
use crate::experiment::ffi::{create_basic_sub, print_basic_sub, test_func};

// TODO: BUGS / FEATURES
// - empty lines are thinner than lines with content by like 1 or 2 pixels :sob:
// - Linux freezes when going from 10000 lines of error to 10000 lines having a value, but updating a value seems ok?
// - Should pre-compile qml files
// - No error handling
// - Ability to remove lines (current, and maybe trim trailing blank lines?)
// - Ctrl + backspace should delete up to cursor rather than whole line
// - Autocomplete in grey, with tab to complete
// - Restore session on startup
// - Should support setting cursor to end of line by clicking on far right when theres no answers configured
// - Seperate colors for keyword vs idents
// - typing in a cell when out of view does not work

fn main() {
    // Client::start();
    test_func();

    let sub = create_basic_sub(123);
    print_basic_sub(&sub);

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
