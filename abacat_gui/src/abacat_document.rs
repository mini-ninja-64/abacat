use abacat_common::{
    error::LocatableFailure,
    ui::{builtin::basic_theme, theme::Theme},
};
use abacat_eval::document::{Document, ParserEvalPair};
use abacat_parser::{ParsingError, parse};
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QColor, QList, QModelIndex, QPoint, QString, QVariant};
use std::{ops::Range, pin::Pin};

use crate::abacat_document::qobject::AbacatDocumentRole;

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

    #[qenum(AbacatDocument)]
    enum AbacatDocumentRole {
        Expression,
        Answer,
        ErrorRange,
        RenderAsError,
    }

    // unsafe extern "RustQt" {}

    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(usize, current_line, cxx_name = "currentLine")]
        #[base = QAbstractListModel]
        type AbacatDocument = super::AbacatDocumentRust;

        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &AbacatDocument, parent: &QModelIndex) -> i32;

        #[cxx_override]
        fn data(self: &AbacatDocument, index: &QModelIndex, role: i32) -> QVariant;

        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &AbacatDocument) -> QHash_i32_QByteArray;

        #[inherit]
        #[cxx_name = "beginInsertRows"]
        fn begin_insert_rows(
            self: Pin<&mut AbacatDocument>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );

        #[inherit]
        #[cxx_name = "endInsertRows"]
        fn end_insert_rows(self: Pin<&mut AbacatDocument>);

        #[inherit]
        #[cxx_name = "dataChanged"]
        fn data_changed(
            self: Pin<&mut AbacatDocument>,
            top_left: &QModelIndex,
            bottom_right: &QModelIndex,
            roles: &QList_i32,
        );

        #[qinvokable]
        #[cxx_name = "setLine"]
        fn set_line(self: Pin<&mut AbacatDocument>, line: usize);

        #[qinvokable]
        #[cxx_name = "getExpr"]
        fn get_expr(self: &AbacatDocument, index: usize) -> QVariant;

        #[qinvokable]
        #[cxx_name = "setExpr"]
        pub fn set_expr(
            self: Pin<&mut AbacatDocument>,
            line: usize,
            string: &QString,
            parent: &QModelIndex,
        );

        #[qinvokable]
        #[cxx_name = "insertRow"]
        fn insert_row(self: Pin<&mut AbacatDocument>, row: usize, value: &QString);

        // TODO: need a way to force re-render, investigate signals and such
        #[qinvokable]
        #[cxx_name = "backgroundColor"]
        fn background_color(self: &AbacatDocument) -> QColor;

        // TODO: need a way to force re-render, investigate signals and such
        #[qinvokable]
        #[cxx_name = "answerOpacity"]
        fn answer_opacity(self: &AbacatDocument) -> f64;

        // TODO: need a way to force re-render, investigate signals and such or just make a property
        #[qinvokable]
        #[cxx_name = "plainTextColor"]
        fn plain_text_color(self: &AbacatDocument) -> QColor;
    }
}

#[derive(Debug)]
pub enum ErrorHighlight {
    UnknownLocation,
    Location(Range<usize>),
}

impl Into<ErrorHighlight> for Option<Range<usize>> {
    fn into(self) -> ErrorHighlight {
        self.map(|r| ErrorHighlight::Location(r))
            .unwrap_or(ErrorHighlight::UnknownLocation)
    }
}

#[derive(Debug)]
pub struct RowData {
    data: QString,
    answer: Option<QString>,
    error_range: Option<ErrorHighlight>,
}

impl RowData {
    pub fn new(data: QString) -> RowData {
        RowData {
            data,
            answer: None,
            error_range: None,
        }
    }
}

pub struct AbacatDocumentRust {
    theme: Theme,
    pub current_line: usize,
    list: Vec<RowData>,
    pub document: Document,
}

impl Default for AbacatDocumentRust {
    fn default() -> Self {
        let doc = Document::new_with_default_constants().with_expr(Err(ParsingError::Empty(0..0)));
        let row_data = vec![RowData::new("".into())];
        // for i in 0..1000 {
        //     let s = format!("{} + ans", i);
        //     doc = doc.with_expr(parse(s.as_str()));
        //     row_data.push(RowData::new(s.into()));
        // }
        Self {
            theme: basic_theme(),
            current_line: 0,
            list: row_data,
            document: doc,
        }
    }
}

impl qobject::AbacatDocument {
    pub fn refresh_rows_cache(mut self: Pin<&mut Self>, from: usize) {
        for i in from..self.list.len() {
            let pair = &self.document.history_at(i).unwrap();

            let (answer, error) = match pair {
                ParserEvalPair(Err(ParsingError::Empty(_)), _) => (None, None),
                // TODO: Extract proper info out of parsing error, make helper to get first error and span
                ParserEvalPair(Err(err), _) => {
                    (Some(err.message().into()), Some(err.span().into()))
                }
                ParserEvalPair(Ok(_), Err(err)) => {
                    (Some(err.message().into()), Some(err.span().into()))
                }
                ParserEvalPair(Ok(_), Ok(eval)) => (Some(eval.value().to_string().into()), None),
            };

            let row = &mut self.as_mut().rust_mut().list[i];
            row.answer = answer;
            row.error_range = error;
        }
    }

    fn row_count(&self, _parent: &QModelIndex) -> i32 {
        self.document.history_len() as i32
    }

    fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let element_role = AbacatDocumentRole { repr: role };
        self.list
            .get(index.row() as usize)
            .and_then(|row| match element_role {
                AbacatDocumentRole::Expression => Some(Into::<QVariant>::into(&row.data)),
                AbacatDocumentRole::Answer => {
                    row.answer.as_ref().map(|ans| Into::<QVariant>::into(ans))
                }
                AbacatDocumentRole::ErrorRange => {
                    row.error_range.as_ref().map(|highglight| match highglight {
                        ErrorHighlight::Location(range) => Into::<QVariant>::into(&QPoint::new(
                            range.start as i32,
                            range.end as i32,
                        )),
                        ErrorHighlight::UnknownLocation => QVariant::default(),
                    })
                }
                AbacatDocumentRole::RenderAsError => {
                    Some(Into::<QVariant>::into(&row.error_range.is_some()))
                }
                _ => unreachable!("This should never happen"),
            })
            .unwrap_or_default()
    }

    fn role_names(&self) -> qobject::QHash_i32_QByteArray {
        let mut hash = qobject::QHash_i32_QByteArray::default();
        hash.insert(AbacatDocumentRole::Expression.repr, "expression".into());
        hash.insert(AbacatDocumentRole::Answer.repr, "answer".into());
        hash.insert(AbacatDocumentRole::ErrorRange.repr, "errorRange".into());
        hash.insert(
            AbacatDocumentRole::RenderAsError.repr,
            "renderAsError".into(),
        );
        hash
    }

    pub fn set_expr(
        mut self: Pin<&mut Self>,
        line: usize,
        new_data: &QString,
        parent: &QModelIndex,
    ) {
        let current_row = &self.list[line];
        // println!("current_row: '{:?}', new_data: '{}'", current_row, new_data);
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
            self_mut.document.replace_at(line, parsed);

            let current = &mut self.as_mut().rust_mut();
            current.list[line] = RowData::new(new_data.clone());
        }
        self.as_mut().refresh_rows_cache(line);

        // TODO: Move to common ref
        let roles: QList<i32> = vec![
            AbacatDocumentRole::Answer.repr,
            AbacatDocumentRole::ErrorRange.repr,
            AbacatDocumentRole::RenderAsError.repr,
        ]
        .into();
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

        let parsed = parse(new_row.to_string().as_str());
        let self_mut = &mut self.as_mut().rust_mut();
        self_mut.document.insert_at(row, parsed);
        self_mut.list.insert(row, RowData::new(new_row.clone()));

        self.as_mut().refresh_rows_cache(0);
        self.as_mut().end_insert_rows();
    }

    pub fn background_color(self: &qobject::AbacatDocument) -> QColor {
        let bg = &self.theme.design.background;
        QColor::from_rgb(bg.red as i32, bg.green as i32, bg.blue as i32)
    }
    pub fn answer_opacity(self: &qobject::AbacatDocument) -> f64 {
        self.theme.design.answer_opacity
    }

    pub fn plain_text_color(self: &qobject::AbacatDocument) -> QColor {
        let color = &self.theme.design.plain_text;
        QColor::from_rgb(color.red as i32, color.green as i32, color.blue as i32)
    }
}
