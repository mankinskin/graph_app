pub fn add(
    a: i32,
    b: i32,
) -> i32 {
    a + b
}

pub fn subtract(
    a: i32,
    b: i32,
) -> i32 {
    a - b
}

pub fn calculate(
    operation: &str,
    a: i32,
    b: i32,
) -> i32 {
    match operation {
        "add" => add(a, b),
        "sub" => subtract(a, b),
        _ => 0,
    }
}

// Nested module
pub mod advanced {
    pub fn multiply(
        a: i32,
        b: i32,
    ) -> i32 {
        a * b
    }

    pub struct Calculator {
        pub result: i32,
    }
}
