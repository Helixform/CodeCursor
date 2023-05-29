use crate::services::ServiceMethod;

pub struct StreamGenerate;

impl ServiceMethod for StreamGenerate {
    fn name(&self) -> &str {
        "StreamGenerate"
    }

    fn kind(&self) -> i32 {
        1
    }
}
