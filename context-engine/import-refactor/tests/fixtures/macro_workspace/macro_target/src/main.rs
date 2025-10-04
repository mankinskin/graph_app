use macro_source::{
    debug_function,
    debug_print,
    MacroHelper,
};

fn main() {
    debug_print!("Hello from macro!");

    let helper = MacroHelper::new(100);
    println!("Helper value: {}", helper.value);

    println!("{}", debug_function());
}
