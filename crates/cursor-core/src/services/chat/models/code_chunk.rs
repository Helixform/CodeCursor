use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct CodeChunk {
    #[serde(rename = "relativeWorkspacePath")]
    pub relative_workspace_path: String,

    #[serde(rename = "startLineNumber")]
    pub start_line: usize,

    pub lines: Vec<String>,
}
