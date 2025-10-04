use macro_source::{MacroHelper, debug_function};

fn main() {
    debug_print!("Hello from macro!");
    
    let helper = MacroHelper::new(100);
    println!("Helper value: {}", helper.value);
    
    println!("{}", debug_function());
}