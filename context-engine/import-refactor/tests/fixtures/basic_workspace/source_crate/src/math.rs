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

// Nested module with deeper nesting
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

    // Even deeper nesting
    pub mod scientific {
        pub fn power(
            base: f64,
            exp: f64,
        ) -> f64 {
            base.powf(exp)
        }

        pub fn logarithm(
            value: f64,
            base: f64,
        ) -> f64 {
            value.log(base)
        }

        pub struct AdvancedCalculator {
            pub precision: u8,
            pub memory: Vec<f64>,
        }

        // Triple nested module
        pub mod statistics {
            pub fn mean(values: &[f64]) -> f64 {
                values.iter().sum::<f64>() / values.len() as f64
            }

            pub fn variance(values: &[f64]) -> f64 {
                let mean = mean(values);
                values.iter().map(|x| (x - mean).powi(2)).sum::<f64>()
                    / values.len() as f64
            }

            pub struct StatEngine {
                pub samples: Vec<f64>,
            }
        }
    }

    // Another nested module at the same level
    pub mod geometry {
        pub fn area_circle(radius: f64) -> f64 {
            std::f64::consts::PI * radius * radius
        }

        pub struct Point {
            pub x: f64,
            pub y: f64,
        }

        pub struct Rectangle {
            pub width: f64,
            pub height: f64,
        }
    }
}

// New top-level nested module
pub mod operations {
    pub fn factorial(n: u64) -> u64 {
        if n <= 1 {
            1
        } else {
            n * factorial(n - 1)
        }
    }

    pub mod matrix {
        pub type Matrix2D = Vec<Vec<f64>>;

        pub fn transpose(matrix: &Matrix2D) -> Matrix2D {
            if matrix.is_empty() {
                return Vec::new();
            }

            let rows = matrix.len();
            let cols = matrix[0].len();
            let mut result = vec![vec![0.0; rows]; cols];

            for i in 0..rows {
                for j in 0..cols {
                    result[j][i] = matrix[i][j];
                }
            }
            result
        }

        pub struct MatrixProcessor {
            pub data: Matrix2D,
        }
    }
}
