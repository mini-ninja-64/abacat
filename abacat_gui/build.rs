use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    let module = QmlModule::new("zone.minis.abacat").version(1, 0);
    CxxQtBuilder::new_qml_module(module)
        .qt_module("Qml")
        .qt_module("Gui")
        .qrc("./qml/qml.qrc")
        .files([
            "src/abacat_document.rs",
            "src/syntax_highlighter.rs",
            "src/qtextcharformat.rs",
        ])
        .cpp_files([
            "./include/qtextcharformat.hpp",
            "./include/syntax_highlighter.hpp",
            "./include/utils.hpp",
        ])
        .build();
}
