use abacat_common::ui::{builtin::basic_theme, theme::Theme};
use abacat_eval::{document::Document, eval::Eval};
use abacat_parser::parse;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QColor, QList, QModelIndex, QString, QVariant};
use qobject::*;
use std::pin::Pin;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!(< QAbstractListModel >);
        type QAbstractListModel;

        include!(< QAbstractTableModel >);
        type QAbstractTableModel;

        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;

        include!("cxx-qt-lib/qmodelindex.h");
        type QModelIndex = cxx_qt_lib::QModelIndex;

        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;

        include!(< QMimeData >);
        type QMimeData;

        include!("cxx-qt-lib/qhash.h");
        type QHash_i32_QByteArray = cxx_qt_lib::QHash<cxx_qt_lib::QHashPair_i32_QByteArray>;

        include!("cxx-qt-lib/qlist.h");
        type QList_i32 = cxx_qt_lib::QList<i32>;

        include!(< QColor >);
        type QColor = cxx_qt_lib::QColor;
    }

    #[qenum(MyObject)]
    enum MyElementRole {
        Expression,
        Answer,
    }

    // unsafe extern "RustQt" {}

    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(usize, current_line, cxx_name = "currentLine")]
        #[base = QAbstractListModel]
        type MyObject = super::MyObjectRust;

        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &MyObject, parent: &QModelIndex) -> i32;

        #[cxx_override]
        fn data(self: &MyObject, index: &QModelIndex, role: i32) -> QVariant;

        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &MyObject) -> QHash_i32_QByteArray;

        #[inherit]
        #[cxx_name = "beginInsertRows"]
        fn begin_insert_rows(self: Pin<&mut MyObject>, parent: &QModelIndex, first: i32, last: i32);

        #[inherit]
        #[cxx_name = "endInsertRows"]
        fn end_insert_rows(self: Pin<&mut MyObject>);

        #[inherit]
        #[cxx_name = "dataChanged"]
        fn data_changed(
            self: Pin<&mut MyObject>,
            top_left: &QModelIndex,
            bottom_right: &QModelIndex,
            roles: &QList_i32,
        );

        #[qinvokable]
        #[cxx_name = "setLine"]
        fn set_line(self: Pin<&mut MyObject>, line: usize);

        #[qinvokable]
        #[cxx_name = "getExpr"]
        fn get_expr(self: &MyObject, index: usize) -> QVariant;

        #[qinvokable]
        #[cxx_name = "setExpr"]
        pub fn set_expr(
            self: Pin<&mut MyObject>,
            line: usize,
            string: &QString,
            parent: &QModelIndex,
        );

        #[qinvokable]
        #[cxx_name = "insertRow"]
        fn insert_row(self: Pin<&mut MyObject>, row: usize, value: &QString);

        // TODO: need a way to force re-render, investigate signals and such
        #[qinvokable]
        #[cxx_name = "backgroundColor"]
        fn background_color(self: &MyObject) -> QColor;

        // TODO: need a way to force re-render, investigate signals and such
        #[qinvokable]
        #[cxx_name = "answerOpacity"]
        fn answer_opacity(self: &MyObject) -> f64;

        // TODO: need a way to force re-render, investigate signals and such or just make a property
        #[qinvokable]
        #[cxx_name = "plainTextColor"]
        fn plain_text_color(self: &MyObject) -> QColor;
    }
}

#[derive(Debug)]
pub struct RowData {
    data: QString,
    answer: Option<QString>,
}
impl RowData {
    pub fn new(data: QString) -> RowData {
        RowData { data, answer: None }
    }
}

pub struct MyObjectRust {
    theme: Theme,
    pub current_line: usize,
    list: Vec<RowData>,
    pub document: Document,
}

impl Default for MyObjectRust {
    fn default() -> Self {
        let mut doc = Document::new_with_default_constants();
        let mut rows = vec![];
        let content = "ans + 1";
        let expr = parse(content).map_err(|_| ());
        for i in 0..10000 {
            doc = doc.with_expr(expr.clone());
            rows.push(RowData::new(content.into()));
        }
        Self {
            theme: basic_theme(),
            current_line: 0,
            // list: vec![RowData::new("".into())],
            // document: Document::new_with_default_constants().with_expr(Err(())),
            list: rows,
            document: doc,
        }
    }
}

impl qobject::MyObject {
    pub fn refresh_rows_cache(mut self: Pin<&mut Self>, from: usize) {
        for i in from..self.list.len() {
            let (_, result) = &self.document.history_at(i).unwrap();

            let ans: Option<QString> = match result {
                Ok(eval) => Some(format!("{}", eval.value()).into()),
                Err(_) => None,
            };

            self.as_mut().rust_mut().list[i].answer = ans;
        }
    }

    fn row_count(&self, _parent: &QModelIndex) -> i32 {
        self.document.history_len() as i32
    }

    fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let element_role = MyElementRole { repr: role };
        self.list
            .get(index.row() as usize)
            .and_then(|row| match element_role {
                MyElementRole::Expression => Some(Into::<QVariant>::into(&row.data)),
                MyElementRole::Answer => row.answer.as_ref().map(|ans| Into::<QVariant>::into(ans)),
                _ => unreachable!("This should never happen"),
            })
            .unwrap_or_default()
    }

    fn role_names(&self) -> QHash_i32_QByteArray {
        let mut hash = QHash_i32_QByteArray::default();
        hash.insert(MyElementRole::Expression.repr, "expression".into());
        hash.insert(MyElementRole::Answer.repr, "answer".into());
        hash
    }

    pub fn set_expr(
        mut self: Pin<&mut Self>,
        line: usize,
        new_data: &QString,
        parent: &QModelIndex,
    ) {
        // println!("row: {}, new_data: {}", line, new_data);
        let current_row = &self.list[line];
        if current_row.data == *new_data {
            return;
        }
        let length = self.document.history_len();

        let parsed = parse(new_data.to_string().as_str());

        if let Ok(expr) = parsed {
            let self_mut = &mut self.as_mut().rust_mut();
            self_mut.document.replace_at(line, Ok(expr.clone()));

            let row_mut = &mut self_mut.list[line];
            row_mut.data = new_data.clone();
        } else {
            let self_mut = &mut self.as_mut().rust_mut();
            self_mut.document.replace_at(line, parsed.map_err(|_| ()));

            let current = &mut self.as_mut().rust_mut();
            current.list[line] = RowData::new(new_data.clone());
        }
        self.as_mut().refresh_rows_cache(line);

        // TODO: Move to common ref
        let roles: QList<i32> = vec![MyElementRole::Answer.repr].into();
        let bottom_right = parent.sibling_at_row((length - 1) as i32);
        self.as_mut().data_changed(&parent, &bottom_right, &roles);
    }

    pub fn get_expr(&self, index: usize) -> QVariant {
        self.list
            .get(index)
            .map(|element| (&element.data).into())
            .unwrap_or_default()
    }

    pub fn set_line(self: Pin<&mut Self>, line: usize) {
        if self.list.len() > line {
            self.set_current_line(line);
        }
    }

    pub fn insert_row(mut self: Pin<&mut Self>, row: usize, new_row: &QString) {
        let parent = QModelIndex::default();
        self.as_mut()
            .begin_insert_rows(&parent, row as i32, row as i32);

        let parsed = parse(new_row.to_string().as_str()).map_err(|_| ());
        let self_mut = &mut self.as_mut().rust_mut();
        self_mut.document.insert_at(row, parsed);
        self_mut.list.insert(row, RowData::new(new_row.clone()));

        self.as_mut().refresh_rows_cache(0);
        self.as_mut().end_insert_rows();
    }

    pub fn background_color(self: &MyObject) -> QColor {
        let bg = &self.theme.design.background;
        QColor::from_rgb(bg.red as i32, bg.green as i32, bg.blue as i32)
    }
    pub fn answer_opacity(self: &MyObject) -> f64 {
        self.theme.design.answer_opacity
    }

    pub fn plain_text_color(self: &MyObject) -> QColor {
        let color = &self.theme.design.plain_text;
        QColor::from_rgb(color.red as i32, color.green as i32, color.blue as i32)
    }
}
