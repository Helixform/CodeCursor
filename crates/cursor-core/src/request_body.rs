use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    Edit,
    Generate,
}

#[derive(Debug, Serialize, Clone)]
pub struct RequestBody {
    #[serde(rename = "userRequest")]
    pub user_request: UserRequest,

    #[serde(rename = "botMessages")]
    pub bot_messages: Vec<BotMessage>,

    #[serde(rename = "userMessages")]
    pub user_messages: Vec<String>,

    #[serde(rename = "contextType")]
    pub context_type: String,

    #[serde(rename = "rootPath")]
    pub root_path: Option<String>,
}

impl RequestBody {
    pub fn new(
        user_request: UserRequest,
        bot_messages: Vec<BotMessage>,
        context_type: String,
        root_path: Option<String>,
    ) -> Self {
        Self {
            user_request,
            bot_messages,
            user_messages: vec![],
            context_type,
            root_path,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct UserRequest {
    pub message: String,

    #[serde(rename = "currentRootPath")]
    pub current_root_path: String,

    #[serde(rename = "currentFileName")]
    pub current_file_name: String,

    #[serde(rename = "currentFileContents")]
    pub current_file_contents: String,

    #[serde(rename = "precedingCode")]
    pub preceding_code: Vec<String>,

    #[serde(rename = "suffixCode")]
    pub suffix_code: Vec<String>,

    #[serde(rename = "currentSelection")]
    pub current_selection: Option<String>,

    #[serde(rename = "copilotCodeBlocks")]
    pub copilot_code_blocks: Vec<String>,

    #[serde(rename = "customCodeBlocks")]
    pub custom_code_blocks: Vec<String>,

    #[serde(rename = "codeBlockIdentifiers")]
    pub code_block_identifiers: Vec<String>,

    #[serde(rename = "msgType")]
    pub message_type: MessageType,

    #[serde(rename = "maxOrigLine")]
    pub max_original_line: i32,
}

impl UserRequest {
    pub fn new(
        message: String,
        current_root_path: String,
        current_file_name: String,
        current_file_contents: String,
        preceding_code: Vec<String>,
        suffix_code: Vec<String>,
        current_selection: Option<String>,
        copilot_code_blocks: Vec<String>,
        custom_code_blocks: Vec<String>,
        message_type: MessageType,
        max_original_line: i32,
    ) -> Self {
        Self {
            message,
            current_root_path,
            current_file_name,
            current_file_contents,
            preceding_code,
            suffix_code,
            current_selection,
            copilot_code_blocks,
            custom_code_blocks,
            code_block_identifiers: vec![],
            message_type,
            max_original_line,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct BotMessage {
    #[serde(rename = "sender")]
    pub sender: String,

    #[serde(rename = "sendAt")]
    pub send_at: i64,

    #[serde(rename = "conversationId")]
    pub conversation_id: String,

    #[serde(rename = "type")]
    pub message_type: String,

    #[serde(rename = "message")]
    pub message: String,

    #[serde(rename = "lastToken")]
    pub last_token: String,

    #[serde(rename = "finished")]
    pub finished: bool,

    #[serde(rename = "currentFile")]
    pub current_file: String,

    #[serde(rename = "interrupted")]
    pub interrupted: bool,

    #[serde(rename = "maxOrigLine")]
    pub max_original_line: i32,

    #[serde(rename = "hitTokenLimit")]
    pub hit_token_limit: bool,
}

impl BotMessage {
    pub fn new(
        sender: String,
        send_at: i64,
        conversation_id: String,
        message_type: String,
        message: String,
        last_token: String,
        finished: bool,
        current_file: String,
        interrupted: bool,
        max_original_line: i32,
        hit_token_limit: bool,
    ) -> Self {
        Self {
            sender,
            send_at,
            conversation_id,
            message_type,
            message,
            last_token,
            finished,
            current_file,
            interrupted,
            max_original_line,
            hit_token_limit,
        }
    }
}
