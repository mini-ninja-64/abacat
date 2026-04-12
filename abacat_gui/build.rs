use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    let module = QmlModule::new("zone.minis.abacat").version(1, 0);
    CxxQtBuilder::new_qml_module(module)
        .qt_module("Qml")
        .qt_module("Gui")
        .qrc("./qml/qml.qrc")
        .files([
            "src/cxxqt_object.rs",
            "src/syntax_highlighter.rs",
            "src/experiment.rs",
        ])
        .cpp_files([
            "./include/experiment.hpp",
            "./include/helper.h",
            "./include/experiment.cpp",
        ])
        .build();
}
