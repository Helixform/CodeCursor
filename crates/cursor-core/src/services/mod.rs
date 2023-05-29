pub mod ai;

pub trait Service {
    fn name(&self) -> &str;
}

pub trait ServiceMethod {
    fn name(&self) -> &str;

    fn kind(&self) -> i32;
}
