pub mod math;
pub mod utils;

pub use math::calculate;

pub struct Config {
    pub name: String,
    pub value: i32,
}

pub fn main_function() -> String {
    "Hello from source".to_string()
}

pub const MAGIC_NUMBER: i32 = 42;

pub static GLOBAL_STATE: &str = "initialized";

pub enum Status {
    Ready,
    Processing,
    Done,
}

pub trait Processable {
    fn process(&self) -> String;
}

// Private items that shouldn't be exported
fn private_function() -> i32 {
    123
}

struct PrivateStruct {
    data: String,
}
