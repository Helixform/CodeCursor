use crate::GenerateInput;

#[derive(Debug, Clone)]
pub struct CodeArea {
    pub preceding_code: String,
    pub following_code: String,
    pub selection_text: Option<String>,
}

impl CodeArea {
    pub fn new_with_input(input: &GenerateInput) -> Self {
        let selection = input.selection_range();
        let document_text_utf16: Vec<u16> = input.document_text().encode_utf16().collect();

        let selection_text = if selection.length() > 0 {
            Some(String::from_utf16_lossy(
                &document_text_utf16[selection.offset()..selection.offset() + selection.length()],
            ))
        } else {
            None
        };
        let preceding_code = String::from_utf16_lossy(&document_text_utf16[0..selection.offset()]);
        let following_code = String::from_utf16_lossy(
            &document_text_utf16[selection.offset() + selection.length()..],
        );

        Self {
            preceding_code,
            following_code,
            selection_text,
        }
    }
}
