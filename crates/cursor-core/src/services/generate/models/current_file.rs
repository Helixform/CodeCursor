use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Selection {
    #[serde(rename = "startPosition")]
    pub start: Position,
    #[serde(rename = "endPosition")]
    pub end: Position,
}

impl Selection {
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start: Position { line: start },
            end: Position { line: end },
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Position {
    pub line: usize,
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
