use source_crate::{Config, main_function, MAGIC_NUMBER};
use source_crate::math::{add, subtract};
use source_crate::math::advanced::Calculator;
use source_crate::utils::format_string;
use source_crate::{Status, Processable};

fn main() {
    let config = Config {
        name: "test".to_string(),
        value: MAGIC_NUMBER,
    };
    
    println!("{}", main_function());
    println!("Sum: {}", add(1, 2));
    println!("Diff: {}", subtract(5, 3));
    println!("{}", format_string("hello"));
    
    let calc = Calculator { result: 0 };
    println!("Calculator result: {}", calc.result);
    
    let status = Status::Ready;
    match status {
        Status::Ready => println!("Ready"),
        Status::Processing => println!("Processing"),
        Status::Done => println!("Done"),
    }
}