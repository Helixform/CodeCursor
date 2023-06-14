use serde::Serialize;

use crate::{Position as IPosition, SelectionRange};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Selection {
    #[serde(rename = "startPosition")]
    pub start: Position,
    #[serde(rename = "endPosition")]
    pub end: Position,
}

impl From<SelectionRange> for Selection {
    fn from(value: SelectionRange) -> Self {
        Self {
            start: value.start().into(),
            end: value.end().into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Position {
    #[serde(skip_serializing_if = "column_is_zero")]
    pub line: usize,
    #[serde(skip_serializing_if = "column_is_zero")]
    pub column: usize,
}

impl From<IPosition> for Position {
    fn from(value: IPosition) -> Self {
        Self {
            line: value.line(),
            column: value.character(),
        }
    }
}

fn column_is_zero(column: &usize) -> bool {
    *column == 0
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentFile {
    #[serde(rename = "contents")]
    pub content: String,
    pub language_id: String,
    pub relative_workspace_path: String,
    pub selection: Selection,
    #[serde(rename = "cursorPosition")]
    pub cursor: Position,
}
