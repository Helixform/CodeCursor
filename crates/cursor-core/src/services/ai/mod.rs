pub mod stream_generate;

pub struct AiService;

impl Service for AiService {
    fn name(&self) -> &str {
        "aiserver.v1.AiService"
    }
}
