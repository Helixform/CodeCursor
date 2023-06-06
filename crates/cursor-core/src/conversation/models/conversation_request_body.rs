use node_bridge::prelude::console;
use serde::Serialize;
use crate::GenerateInput;

use super::{UserRequest, bot_message::BotMessage, user_message::UserMessage};

#[derive(Debug, Serialize, Clone)]
pub struct CursorPosition {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Selection {
    pub start_position: CursorPosition,
    pub end_position: CursorPosition,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CurrentFile {
    pub relative_workspace_path: String,
    pub contents: String,
    pub selection: Selection,
    pub cursor_position: CursorPosition,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_id: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModelDetails {
    pub model_name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConversationMessageType {
    MessageTypeHuman,
    MessageTypeAi,
}

#[derive(Debug, Serialize, Clone)]
pub struct Conversation {
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,

    #[serde(rename = "type")]
    message_type: ConversationMessageType,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConversationRequestBody {
    current_file: CurrentFile,
    conversation: Vec<Conversation>,
    workspace_root_path: String,
    pub model_details: ModelDetails,
}

impl ConversationRequestBody {
    /// encoded as envelop message
    /// 
    /// https://connect.build/docs/protocol
    pub fn to_enveloped_message(&self) -> Vec<u8> {
        let json_message = serde_json::to_string(self).unwrap_or("".to_string());
        let bytes_message = json_message.into_bytes();
        let bytes_message_length = bytes_message.len();

        let mut enveloped_message = Vec::<u8>::new();

         // type    payload_length  payload
         // 1byte      4byte        
         //
         // when type = 0, payload is the regular message
         // when type = 2, payload is the final message    
        {
            enveloped_message.push(0);
            let bytes_message_length: u32 = bytes_message_length.try_into().unwrap();
            let mut offset = 24;
            let mut mask = 255u32 << offset;
            loop {
                if mask == 0 {
                    break
                }
                let v:u8 = ((bytes_message_length & mask) >> offset).try_into().unwrap_or(0);
                enveloped_message.push(v);
                mask = mask >> 8;
                offset -= 8;
            }

            enveloped_message.extend_from_slice(&bytes_message);
        }

        enveloped_message.extend_from_slice(&[2, 0, 0, 0, 0]);

        enveloped_message

    }
}

impl From<&GenerateInput> for ConversationRequestBody {
    fn from(input: &GenerateInput) -> Self {
        let selection = Selection {
            start_position: CursorPosition { 
                line:  input.selection_range().start_line(), 
                column: input.selection_range().start_column(),
            },
            end_position: CursorPosition { 
                line: input.selection_range().end_line(), 
                column: input.selection_range().end_column(),
            },
        };

        let cursor_position = CursorPosition {
            line: input.selection_range().start_line(),
            column: input.selection_range().start_column(),
        };

        let current_file = CurrentFile {
            selection,
            cursor_position,
            relative_workspace_path: input.file_path(),
            contents: input.document_text(),
            language_id: None,
        };

        let workspace_root_path = input.file_dir();

        let model_details = ModelDetails {
            model_name: input.gpt_model().unwrap_or("gpt-3.5-turbo".to_string()),
            api_key: input.api_key(),
        };

        let mut conversation = Vec::<Conversation>::new();
        conversation.extend_from_slice(&[
            Conversation { 
                text: Some(input.prompt()), 
                message_type: ConversationMessageType::MessageTypeHuman
            },
            Conversation {
                text: None,
                message_type: ConversationMessageType::MessageTypeAi,
            }
        ]);
       
        Self {
            current_file,
            workspace_root_path,
            model_details,
            conversation
        }
    }
}
